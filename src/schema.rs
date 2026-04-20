// @generated automatically by Diesel CLI.

diesel::table! {
    app_channel (id) {
        id -> Uuid,
        channel_name -> Varchar,
        create_user_id -> Uuid,
        create_time -> Timestamp,
        update_time -> Timestamp,
        is_delete -> Bool,
    }
}

diesel::table! {
    app_manage (id) {
        id -> Uuid,
        app_name -> Varchar,
        app_download_url -> Varchar,
        create_user_id -> Uuid,
        channel_id -> Uuid,
        create_time -> Timestamp,
        update_time -> Timestamp,
        is_delete -> Bool,
        file_path -> Nullable<Varchar>,
        file_name -> Nullable<Varchar>,
        package_name -> Nullable<Varchar>,
        app_icon_path -> Nullable<Varchar>,
        version_name -> Nullable<Varchar>,
        version_code -> Varchar,
        file_size -> Int8,
        channel_name -> Nullable<Varchar>,
        update_log -> Nullable<Varchar>,
    }
}

diesel::table! {
    operation_log (id) {
        id -> Uuid,
        user_id -> Uuid,
        username -> Varchar,
        operation_type -> Varchar,
        operation_detail -> Varchar,
        create_time -> Timestamp,
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
        update_time -> Timestamp,
        is_delete -> Bool,
    }
}

diesel::joinable!(app_channel -> users (create_user_id));
diesel::joinable!(app_manage -> app_channel (channel_id));
diesel::joinable!(app_manage -> users (create_user_id));
diesel::joinable!(operation_log -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(app_channel, app_manage, operation_log, users,);
