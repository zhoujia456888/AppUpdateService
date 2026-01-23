// @generated automatically by Diesel CLI.

diesel::table! {
    app_channel (id) {
        id -> Uuid,
        channel_name -> Varchar,
        create_user_id -> Nullable<Uuid>,
        create_time -> Timestamp,
        is_delete -> Bool,
    }
}

diesel::table! {
    app_manage (id) {
        id -> Uuid,
        app_name -> Varchar,
        app_version -> Varchar,
        app_download_url -> Nullable<Varchar>,
        create_user_id -> Nullable<Uuid>,
        channel_id -> Nullable<Uuid>,
        create_time -> Timestamp,
        is_delete -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        password -> Varchar,
        full_name -> Varchar,
        access_token -> Varchar,
        refresh_token -> Varchar,
        create_time -> Timestamp,
        is_delete -> Bool,
    }
}

diesel::joinable!(app_channel -> users (create_user_id));
diesel::joinable!(app_manage -> app_channel (channel_id));
diesel::joinable!(app_manage -> users (create_user_id));

diesel::allow_tables_to_appear_in_same_query!(app_channel, app_manage, users,);
