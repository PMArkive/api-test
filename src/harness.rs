use demostf_client::ApiClient;
use sqlx::{Pool, Postgres};
use color_eyre::Result;
use sqlx::postgres::PgPoolOptions;

pub struct Harness {
    client: ApiClient,
    db: Pool<Postgres>,
}

impl Harness {
    pub async fn new(base_url: &str, db_url: &str) -> Result<Self> {
        let client = ApiClient::with_base_url(base_url)?;
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .unwrap();

        Ok(Harness {
            client,
            db,
        })
    }

    pub async fn reset(&self) -> Result<()> {
        let tables = [
            "chat",
            "demos",
            "players",
            "storage_keys",
            "teams",
            "upload_blacklist",
            "users",
        ];

        let mut transaction = self.db.begin().await?;

        for table in &tables {
            sqlx::query(&format!("TRUNCATE TABLE {}", table))
                .execute(&mut transaction)
                .await?;
            sqlx::query(&format!("ALTER SEQUENCE {}_id_seq RESTART with 1", table))
                .execute(&mut transaction)
                .await?;
        }

        sqlx::query("INSERT INTO users(steamid, name, avatar, token)\
    VALUES(76561198024494988, 'Icewind', 'http://cdn.akamai.steamstatic.com/steamcommunity/public/images/avatars/75/75b84075b70535c5cfb3499af03b3e4e7a7b556f_medium.jpg', 'token')")
            .execute(&mut transaction).await?;

        transaction.commit().await?;
        Ok(())
    }

    pub fn client(&self) -> ApiClient {
        self.client.clone()
    }
}