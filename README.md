# 🚀 Social Media Backend

A high-performance, asynchronous **Rust** backend for a social media platform built with **Rocket**, **SurrealDB**, and **WebSockets**.
This service provides user authentication, social graph relationships, posts with media, likes, and real-time private messaging between users.

---

# 📌 Overview

This backend is designed with performance, safety, and scalability in mind. It leverages Rust’s memory safety and async concurrency model to deliver a fast and reliable API server capable of handling real-time communication and high-volume interactions.

### Core Capabilities

* User registration and authentication
* JWT access + refresh token security
* Follow system
* Media post creation with captions
* Like system
* Real-time 1-to-1 chat
* Centralized error handling
* Async shared state management

---

# 🧠 Architecture

The system follows a modular architecture separating responsibilities into distinct layers:

```
Client → Routes → Services → Database → Response
```

### Layers

| Layer    | Responsibility               |
| -------- | ---------------------------- |
| Routes   | HTTP + WebSocket entrypoints |
| Services | Business logic               |
| DB       | SurrealDB queries            |
| Models   | Typed data structures        |
| Auth     | JWT handling                 |
| Errors   | Unified error types          |

---

# ⚙️ Core Technologies

| Component      | Stack     |
| -------------- | --------- |
| Language       | Rust      |
| Framework      | Rocket    |
| Database       | SurrealDB |
| WebSocket      | rocket_ws |
| Async Runtime  | Tokio     |
| Auth           | JWT       |
| Error Handling | thiserror |
| Concurrency    | Arc       |

---

# 📂 Project Structure

```
src/
├── auth/        → authentication logic
├── db/          → database connection + queries
├── users/       → user features
├── posts/       → post logic
├── chat/        → websocket messaging
├── ws/          → socket manager
├── errors/      → app error definitions
└── main.rs      → application entry
```

---

# 🔐 Authentication System

Authentication is handled through JWT tokens.

### Token Types

| Token         | Purpose                            |
| ------------- | ---------------------------------- |
| Access Token  | Used for API authentication        |
| Refresh Token | Used to generate new access tokens |

### Flow

```
Login/Register
      ↓
Issue Tokens
      ↓
Access Protected Routes
      ↓
Refresh Token When Expired
```

Security principles:

* Signed tokens
* Expiration enforcement
* Server validation guard

---

# 🗄 Database Schema

The backend uses structured SurrealDB tables with constraints, indexes, and relations.

---

## 👤 users

Stores account information.

| Field           | Type           | Notes               |
| --------------- | -------------- | ------------------- |
| email           | string         | must be valid email |
| username        | string         | unique              |
| password_hash   | string         | required            |
| mobile_number   | string         | unique              |
| profile_picture | option<string> | nullable            |
| followers_count | int            | ≥ 0                 |
| following_count | int            | ≥ 0                 |

Indexes:

* unique email
* unique username
* unique mobile number

---

## 🤝 follows

Represents follow relationships.

| Field        | Type          |
| ------------ | ------------- |
| follower_id  | record<users> |
| following_id | record<users> |
| created_at   | datetime      |

Indexes:

* follower_id index
* following_id index
* unique pair constraint

---

## 📝 posts

Stores user posts.

| Field       | Type          |
| ----------- | ------------- |
| uid         | record<users> |
| content     | string        |
| caption     | string        |
| created_at  | datetime      |
| likes_count | int           |

Indexes:

* created_at index (optimized feed queries)

---

## ❤️ likes

Tracks likes per post.

| Field    | Type                 |
| -------- | -------------------- |
| post_id  | record<posts>        |
| user_ids | array<record<users>> |

Constraints:

* Unique combination of post_id + user_id

---

## 💬 conversation

Represents a private chat session between two users.

| Field        | Type                 |
| ------------ | -------------------- |
| participants | array<record<users>> |
| pair_key     | string               |
| created_at   | datetime             |

Indexes:

* unique pair_key (ensures only one conversation per user pair)

---

## 📩 message

Stores individual messages.

| Field           | Type                                 |
| --------------- | ------------------------------------ |
| conversation_id | record<conversation>                 |
| sender_id       | record<users>                        |
| text            | string                               |
| status          | string (`SENT`, `DELIVERED`, `SEEN`) |
| created_at      | datetime                             |
| read_at         | option<datetime>                     |

---

# 🔗 Relationships Overview

```
User ──< Follows >── User

User ──< Posts

Post ──< Likes >── Users

Conversation ──< Messages
Conversation ── Participants → Users
```

---

# 💬 Real-Time Chat Design

The chat system uses WebSockets to maintain persistent duplex connections between clients and server.

### Chat Characteristics

* bidirectional messaging
* low latency
* persistent connection
* message state tracking
* conversation-scoped routing

Message lifecycle:

```
Send → Stored → Delivered → Seen
```

---

# ⚡ Performance Design Decisions

This backend is optimized for concurrency and efficiency:

* async request handling
* shared immutable state via `Arc`
* database indexing for hot queries
* minimal allocations
* typed database models

---

# 🧯 Error Handling Strategy

All errors flow through centralized application error types implemented with `thiserror`.

Benefits:

* consistent responses
* clear debugging
* structured error messages
* reduced boilerplate

---

# 📈 Scalability Considerations

The system is designed to scale horizontally and vertically.

Prepared for:

* sharded databases
* distributed websocket nodes
* load balancing
* caching layers
* CDN for media

---

# 🧩 Design Principles

The project follows modern backend engineering standards:

* type safety
* separation of concerns
* explicit state management
* database constraints
* deterministic logic
* minimal runtime overhead

---

**Built for performance, correctness, and real-time scale using Rust.**
