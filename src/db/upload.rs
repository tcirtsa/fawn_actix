// @generated automatically by Diesel CLI.

diesel::table! {
    upload (file_name) {
        #[max_length = 255]
        file_name -> Varchar,
        file_data -> Bytea,
        #[max_length = 255]
        userid -> Varchar,
        #[max_length = 255]
        time -> VarChar,
    }
}
