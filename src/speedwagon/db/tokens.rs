use time::Tm;

#[derive(Insertable)]
#[table_name = "tokens"]
struct InsertableToken {
    id: Uuid,
    user: User,
    expires: Tm
}
