use crate::{
    AppResult, DB,
    common_service::validate_image,
    db::parse_thing,
    error::AppError,
    jwt::{AuthUser, generate_access_token, generate_refresh_token, refresh_access_token},
    users::model::{
        DBUser, Follow, LoginRequest, RefreshRequest, RegisterRequest, Upload, User, UserResponse,
    },
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use rocket::{State, delete, form::Form, get, post, put, serde::json::Json};
use serde_json::{Value, json};

use surrealdb::types::ToSql;
use surrealdb_types::RecordId;
use uuid::Uuid;
use validator::{Validate, ValidationError};

#[post("/login", data = "<req>")]
pub async fn login_request(req: Json<LoginRequest>, db: &State<DB>) -> AppResult<Value> {
    let res = login(db, req.into_inner()).await?;
    Ok(res)
}

async fn login(db: &State<DB>, req: LoginRequest) -> AppResult<Value> {
    req.validate()?;
    if req.email == None && req.username == None {
        return Err(AppError::ValidationError(ValidationError::new(
            "email or username required",
        )));
    }
    let mut response = db
        .query(
            "
            SELECT * FROM users
            WHERE email = $email
            OR username = $username
            LIMIT 1
        ",
        )
        .bind(("email", req.email.clone()))
        .bind(("username", req.username.clone()))
        .await?;
    let user: Option<DBUser> = response.take(0)?;
    let user = user.ok_or(AppError::XCustomMessage("User not found"))?;
    let parsed_hash = PasswordHash::new(&user.password_hash)?;
    Argon2::default().verify_password(req.password.as_bytes(), &parsed_hash)?;
    let token = generate_access_token(user.id.to_sql())?;
    let refresh = generate_refresh_token(user.id.to_sql())?;
    Ok(json!(
        {
        "access_token":token,
        "refresh_token":refresh
        }
    ))
}

#[post("/sign-up", data = "<req>")]
pub async fn register_request(req: Json<RegisterRequest>, db: &State<DB>) -> AppResult<Value> {
    req.validate()?;
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|_| AppError::XCustomMessage("Password hash not found"))?;

    let email = req.email.clone();
    let username = req.username.clone();
    let mobile = req.mobile_number.clone();

    let mut res = db
        .query(
            "
            CREATE users SET
                username = $username,
                email = $email,
                password_hash = $hash,
                mobile_number = $mobile_number
        ",
        )
        .bind(("username", username))
        .bind(("email", email))
        .bind(("hash", password_hash.to_string()))
        .bind(("mobile_number", mobile))
        .await?;
    res.take::<Option<User>>(0)?;
    let res = login(db, req.into_inner().into()).await?;
    Ok(res)
}

#[get("/user-info")]
pub async fn get_user_details_from_token(
    db: &State<DB>,
    auth: AuthUser,
) -> Result<Json<UserResponse>, AppError> {
    let res: Option<User> = db
        .query("SELECT * OMIT password_hash FROM $id")
        .bind(("id", parse_thing(&auth.user_id)?))
        .await?
        .take::<Option<User>>(0)?;
    let user = res.ok_or(AppError::XCustomMessage("User not found"))?;
    Ok(Json(user.into()))
}

#[put("/refresh-token", data = "<req>")]
pub async fn refresh_token(req: Json<RefreshRequest>) -> AppResult<Value> {
    let new_access = refresh_access_token(&req.refresh_token)?;
    Ok(json!({
        "access_token":new_access
    }))
}

#[get("/get-followers")]
pub async fn get_follower_list(db: &State<DB>, auth: AuthUser) -> AppResult<Json<Vec<String>>> {
    let res = db
        .query(
            "
                SELECT VALUE follower_id
                FROM follows
                WHERE following_id = $id
            ",
        )
        .bind(("id", parse_thing(&auth.user_id)?))
        .await?
        .take::<Vec<RecordId>>(0)?;
    let list: Vec<String> = res.into_iter().map(|e| e.to_sql()).collect();
    Ok(Json(list))
}

#[put("/follow-user/<uid>")]
pub async fn follow_user(uid: &str, db: &State<DB>, auth: AuthUser) -> AppResult<String> {
    let mut res = db
        .query(
            "
            BEGIN TRANSACTION;
        CREATE follows SET
            follower_id = $myid,
            following_id = $uid,
            created_at = time::now();
        UPDATE $myid SET following_count += 1;
        UPDATE $uid SET followers_count += 1;
        COMMIT TRANSACTION;
        ",
        )
        .bind(("myid", parse_thing(&auth.user_id)?))
        .bind(("uid", parse_thing(uid)?))
        .await?;
    let user: Follow = res
        .take::<Option<Follow>>(0)?
        .ok_or(AppError::XCustomMessage("Error occured"))?;
    Ok(format!("Followed user : {}", user.follower_id.to_sql()))
}

#[get("/get-following")]
pub async fn get_following_list(auth: AuthUser, db: &State<DB>) -> AppResult<Json<Vec<String>>> {
    let res = db
        .query(
            "
                SELECT VALUE following_id
                FROM follows
                WHERE follower_id = $id
            ",
        )
        .bind(("id", parse_thing(&auth.user_id)?))
        .await?
        .take::<Vec<RecordId>>(0)?;
    let list: Vec<String> = res.into_iter().map(|e| e.to_sql()).collect();
    Ok(Json(list))
}

#[delete("/unfollow-user/<uid>")]
pub async fn unfollow_user(uid: &str, db: &State<DB>, auth: AuthUser) -> AppResult<String> {
    let mut res = db
        .query(
            "
            BEGIN TRANSACTION;
            LET $fid = SELECT * FROM follows WHERE follower_id=$follower && following_id=$following LIMIT 1;
            $fid[0];
            if $fid[0] !=NONE{
                DELETE $fid.id;
                UPDATE $follower SET following_count-=1;
                UPDATE $following SET followers_count-=1;
            };
            COMMIT TRANSACTION;
        ",
        )
        .bind(("follower", parse_thing(&auth.user_id)?))
        .bind(("following", parse_thing(uid)?))
        .await?;
    let _follow: Follow = res
        .take::<Option<Follow>>(1)?
        .ok_or(AppError::XCustomMessage("Cannot Unfollow"))?;
    Ok(format!("Unfollowed user"))
}

#[post(
    "/update-profile-picture",
    data = "<upload>",
    format = "multipart/form-data"
)]
pub async fn update_profile_picture(
    mut upload: Form<Upload<'_>>,
    db: &State<DB>,
    auth: AuthUser,
) -> AppResult<String> {
    let file = &mut upload.file;

    let filename = format!("{}.png", Uuid::new_v4());
    let path = format!("data/profile-pictures/{}", filename);
    let content_type =
        file.content_type()
            .ok_or(AppError::ValidationError(ValidationError::new(
                "Missing content type",
            )))?;

    if !validate_image(content_type) {
        return Err(AppError::ValidationError(ValidationError::new(
            "File must be an image",
        )));
    }
    file.persist_to(&path).await?;
    let url = format!("http://localhost:8080/profile-pictures/{}", filename);
    db.query(
        "
            UPDATE $id
            SET profile_picture = $path
            ",
    )
    .bind(("id", parse_thing(&auth.user_id)?))
    .bind(("path", url.clone()))
    .await?;
    Ok(url)
}
