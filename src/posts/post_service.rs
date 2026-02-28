use rocket::{State, form::Form, get, post, put, serde::json::Json};

use uuid::Uuid;
use validator::ValidationError;

use crate::{
    AppResult, DB,
    common_service::validate_image,
    db::{parse_thing, parse_thing_to_record},
    error::AppError,
    jwt::AuthUser,
    posts::model::{
        FeedQuery, Like, LikeResponse, Post, PostFormRequest, PostRequest, PostResponse,
    },
};

#[post("/post", data = "<form>", format = "multipart/form-data")]
pub async fn post(
    mut form: Form<PostFormRequest<'_>>,
    db: &State<DB>,
    auth: AuthUser,
) -> AppResult<Json<PostResponse>> {
    let caption = form.caption.clone();
    let file = &mut form.content;
    let filename = format!("{}.png", Uuid::new_v4());
    let path = format!("data/posts/{}", filename);
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
    let url = format!("http://localhost:8080/posts/{}", filename);
    let mut res = db
        .create("posts")
        .content(PostRequest {
            caption: caption,
            content: url,
            uid: parse_thing(&auth.user_id)?,
        })
        .await?;
    let post: Post = res
        .take()
        .ok_or(AppError::XCustomMessage("Failed to post"))?;
    Ok(Json(post.into()))
}

#[get("/get-user-posts")]
pub async fn get_user_posts(db: &State<DB>, auth: AuthUser) -> AppResult<Json<Vec<PostResponse>>> {
    let res: Vec<Post> = db
        .query("SELECT * FROM posts WHERE uid=$uid ORDER BY created_at DESC")
        .bind(("uid", parse_thing(&auth.user_id)?))
        .await?
        .take::<Vec<Post>>(0)?;
    let mut posts: Vec<PostResponse> = res.into_iter().map(Into::into).collect();
    for e in posts.iter_mut() {
        let liked = liked_by_user(db, &auth.user_id, &e.id).await?;
        e.liked_by_user = liked;
    }
    Ok(Json(posts))
}

#[get("/feed?<q..>")]
pub async fn get_feed(
    db: &State<DB>,
    auth: AuthUser,
    q: FeedQuery,
) -> AppResult<Json<Vec<PostResponse>>> {
    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(10);
    let start = (page - 1) * limit;
    let res: Vec<Post> = db
        .query(
            "
        SELECT * FROM posts
        WHERE uid IN (
            SELECT VALUE following_id
            FROM follows
            WHERE follower_id = $uid
        )
        ORDER BY created_at DESC
        LIMIT $limit
        START $offset;
        ",
        )
        .bind(("uid", parse_thing(&auth.user_id)?))
        .bind(("limit", limit))
        .bind(("offset", start))
        .await?
        .take(0)?;
    let mut posts: Vec<PostResponse> = res.into_iter().map(Into::into).collect();
    for e in posts.iter_mut() {
        let liked = liked_by_user(db, &auth.user_id, &e.id).await?;
        e.liked_by_user = liked;
    }
    Ok(Json(posts))
}

#[get("/get-post-by-id/<id>")]
pub async fn get_post_by_id(
    id: &str,
    db: &State<DB>,
    auth: AuthUser,
) -> AppResult<Json<PostResponse>> {
    let res: Post = db
        .select(parse_thing_to_record(id)?)
        .await?
        .ok_or(AppError::XCustomMessage("Post not found"))?;
    let mut post: PostResponse = res.into();
    let liked = liked_by_user(db, &auth.user_id, id).await?;
    post.liked_by_user = liked;
    Ok(Json(post))
}

async fn liked_by_user(db: &State<DB>, uid: &str, pid: &str) -> AppResult<bool> {
    let res = db
        .query(
            "
        SELECT VALUE $user_id INSIDE user_ids
        FROM likes
        WHERE post_id = $post_id
        LIMIT 1;
        ",
        )
        .bind(("user_id", parse_thing(uid)?))
        .bind(("post_id", parse_thing(pid)?))
        .await?
        .take::<Option<bool>>(0)?;
    if res.is_some() {
        Ok(res.unwrap())
    } else {
        Ok(false)
    }
}

#[put("/like-post/<id>")]
pub async fn like_post(id: &str, db: &State<DB>, auth: AuthUser) -> AppResult<String> {
    let uid = parse_thing(&auth.user_id)?;
    let pid = parse_thing(id)?;
    let mut res = db
        .query(
            "
            BEGIN TRANSACTION;

            LET $rec = SELECT * FROM likes WHERE post_id=$pid LIMIT 1;
            LET $row = $rec[0];
            IF $row = NONE {
                CREATE likes SET
                    post_id = $pid,
                    user_ids += $uid;
                UPDATE $pid SET likes_count += 1;
            } ELSE {
                LET $users = IF $row.user_ids = NONE THEN [] ELSE $row.user_ids END;
                IF $uid INSIDE $users  {
                    UPDATE $row.id SET user_ids -= $uid;
                    UPDATE $pid SET likes_count -= 1;
                } ELSE {
                    UPDATE $row.id SET user_ids += $uid;
                    UPDATE $pid SET likes_count += 1;
                };

            };
            SELECT * FROM likes WHERE post_id=$pid;
            COMMIT TRANSACTION;
        ",
        )
        .bind(("uid", uid.clone()))
        .bind(("pid", pid))
        .await?;
    let like = res
        .take::<Option<Like>>(3)?
        .ok_or(AppError::XCustomMessage("Error occured"))?;
    if like.user_ids.contains(&uid) {
        return Ok(format!("Liked the post"));
    } else {
        return Ok(format!("Unliked the post"));
    }
}

#[get("/get-likes/<id>")]
pub async fn get_likes(id: &str, db: &State<DB>) -> AppResult<Json<Option<LikeResponse>>> {
    let res = db
        .query("SELECT * FROM likes WHERE post_id=$pid")
        .bind(("pid", parse_thing(id)?))
        .await?
        .take::<Vec<Like>>(0)?;
    let like = res.into_iter().next().map(LikeResponse::from);
    Ok(Json(like))
}
