mod harness;
mod report;

use crate::harness::Harness;
use bitbuffer::{BitReadBuffer, LittleEndian};
use color_eyre::{eyre::WrapErr, Report, Result};
use demostf_client::{Class, Demo, SteamID, Team};
use report::{assert_eq, Test};
use std::str::FromStr;
use tf_demo_parser::DemoParser;

macro_rules! assert_object_eq {
    ($obj:expr => { $($name:ident == $value:expr),* }) => {
        $(report::assert_eq_borrow(&$obj.$name, $value)?;)*
    };
    ($obj:expr => { $($name:ident == $value:expr),* , }) => {
        $(report::assert_eq_borrow(&$obj.$name, $value)?;)*
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    let harness = Harness::new(&dotenv::var("BASE_URL")?, &dotenv::var("DB_URL")?).await?;

    Test::run(
        "Upload demo, then retrieve info",
        &harness,
        |test| async move {
            let id = test
                .step("upload", |client| async move {
                    Ok(client
                        .upload_demo(
                            String::from("test.dem"),
                            std::fs::read("data/gully.dem")?,
                            String::from("RED"),
                            String::from("BLUE"),
                            String::from("token"),
                        )
                        .await?)
                })
                .await?;

            assert_eq(id, 1)?;

            test.step("get", |client| async move {
                let demo = client.get(id).await?;
                assert_object_eq!(demo => {
                    id == 1,
                    name == "test.dem",
                    map == "cp_gullywash_final1",
                    red_score == 5,
                    blue_score == 3,
                    player_count == 12,
                });
                verify_demo(&demo, std::fs::read("data/gully.dem")?)?;
                assert_eq(demo.uploader.id(), 1)?;

                let uploader = demo.uploader.resolve(client).await?;
                assert_eq(&uploader.name, "Icewind")?;

                Ok(())
            })
            .await?;
            Ok(())
        },
    )
    .await;

    Ok(())
}

fn verify_demo(api_result: &Demo, demo: Vec<u8>) -> Result<()> {
    use tf_demo_parser::demo::parser::gamestateanalyser;

    fn map_team(team: Team) -> gamestateanalyser::Team {
        match team {
            Team::Red => gamestateanalyser::Team::Red,
            Team::Blue => gamestateanalyser::Team::Blue,
        }
    }

    fn map_class(class: Class) -> gamestateanalyser::Class {
        match class {
            Class::Scout => gamestateanalyser::Class::Scout,
            Class::Soldier => gamestateanalyser::Class::Soldier,
            Class::Pyro => gamestateanalyser::Class::Pyro,
            Class::Demoman => gamestateanalyser::Class::Demoman,
            Class::HeavyWeapons => gamestateanalyser::Class::Heavy,
            Class::Medic => gamestateanalyser::Class::Medic,
            Class::Engineer => gamestateanalyser::Class::Engineer,
            Class::Sniper => gamestateanalyser::Class::Sniper,
            Class::Spy => gamestateanalyser::Class::Spy,
        }
    }

    let parser = DemoParser::new(BitReadBuffer::new(demo, LittleEndian).into());
    let (header, state) = parser
        .parse()
        .map_err(|_| Report::msg("Failed to parse demo"))?;
    assert_eq(&api_result.map, &header.map).wrap_err("Failed to compare map")?;
    assert_eq(
        api_result.red_score,
        state
            .rounds
            .iter()
            .filter(|round| round.winner == gamestateanalyser::Team::Red)
            .count() as u8,
    )
    .wrap_err("Failed to compare red score")?;
    assert_eq(
        api_result.blue_score,
        state
            .rounds
            .iter()
            .filter(|round| round.winner == gamestateanalyser::Team::Blue)
            .count() as u8,
    )
    .wrap_err("Failed to compare blue score")?;

    let mut players = state
        .users
        .values()
        .filter(|user| user.team.is_player())
        .collect::<Vec<_>>();

    players.sort_by(|a, b| {
        SteamID::from_str(&a.steam_id)
            .unwrap()
            .account_id()
            .cmp(&SteamID::from_str(&b.steam_id).unwrap().account_id())
    });

    let mut api_players = api_result.players.iter().collect::<Vec<_>>();
    api_players.sort_by(|a, b| {
        a.user
            .steam_id
            .account_id()
            .cmp(&b.user.steam_id.account_id())
    });

    assert_eq(api_result.player_count, players.len() as u8)
        .wrap_err("Failed to compare player count")?;
    assert_eq(api_players.len(), players.len()).wrap_err("Failed to compare player count")?;

    for (api_player, player) in api_players.iter().zip(players.iter()) {
        assert_eq(&api_player.user.name, &player.name).wrap_err_with(|| {
            format!("Failed to compare player name for {}", api_player.user.name)
        })?;
        assert_eq(
            &api_player.user.steam_id,
            &SteamID::from_str(&player.steam_id).unwrap(),
        )
        .wrap_err_with(|| format!("Failed to compare steam id for {}", api_player.user.name))?;
        assert_eq(map_team(api_player.team), player.team)
            .wrap_err_with(|| format!("Failed to compare team for {}", api_player.user.name))?;
        assert_eq(
            map_class(api_player.class),
            player.classes.sorted().next().unwrap().0,
        )
        .wrap_err_with(|| format!("Failed to compare class for {}", api_player.user.name))?;
        let kills = state
            .deaths
            .iter()
            .filter(|kill| kill.killer == player.user_id)
            .count() as u8;
        let assists = state
            .deaths
            .iter()
            .filter(|kill| kill.assister == Some(player.user_id))
            .count() as u8;
        let deaths = state
            .deaths
            .iter()
            .filter(|kill| kill.victim == player.user_id)
            .count() as u8;
        assert_eq(api_player.kills, kills)
            .wrap_err_with(|| format!("Failed to compare kills for {}", api_player.user.name))?;
        assert_eq(api_player.assists, assists)
            .wrap_err_with(|| format!("Failed to compare assists for {}", api_player.user.name))?;
        assert_eq(api_player.deaths, deaths)
            .wrap_err_with(|| format!("Failed to compare deaths for {}", api_player.user.name))?;
    }

    Ok(())
}
