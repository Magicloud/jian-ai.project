table! {
    names (name) {
        name -> Text,
    }
}

table! {
    photos (filename) {
        datetime -> Nullable<Timestamp>,
        filename -> Text,
        camera_id -> Text,
        food_weight -> SmallInt,
        name -> Nullable<Text>,
    }
}

joinable!(photos -> names (name));

allow_tables_to_appear_in_same_query!(
    names,
    photos,
);
