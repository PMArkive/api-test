use crate::harness::Harness;
use color_eyre::{Report, Result};
use colored::Colorize;
use demostf_client::ApiClient;
use std::fmt::Debug;
use std::future::Future;

#[derive(Clone)]
pub struct Test {
    client: ApiClient,
}

impl Test {
    pub async fn run<'a, Fut: Future<Output = Result<()>> + 'a, F: FnOnce(Test) -> Fut + 'a>(
        name: &str,
        harness: &'a Harness,
        f: F,
    ) {
        println!(" - {}", name);

        if let Err(e) = harness.reset().await {
            println!("   {}: {:#}", "Reset api server".red(), e);
            println!("      {}", "❌".red());
            return;
        } else {
            println!("    - {}", "Reset api server".green());
        }

        let test = Test {
            client: harness.client(),
        };

        match f(test).await {
            Ok(_) => {
                println!("      {}", "✓".green());
            }
            Err(e) => {
                println!("      {}: {:#}", "❌".red(), e);
            }
        }
    }

    pub async fn step<
        'a,
        T,
        Fut: Future<Output = Result<T>> + 'a,
        F: FnOnce(&'a ApiClient) -> Fut + 'a,
    >(
        &'a self,
        name: &str,
        f: F,
    ) -> Result<T> {
        match f(&self.client).await {
            Ok(res) => {
                println!("    - {}", name.green());
                Ok(res)
            }
            Err(e) => {
                println!("    - {}: {:#}", name.red(), e);
                Err(e)
            }
        }
    }
}

pub fn assert_eq<A: Debug, B: PartialEq<A> + Debug>(a: A, b: B) -> Result<()> {
    if b.eq(&a) {
        Ok(())
    } else {
        Err(Report::msg(format!(
            "Failed asserting that {:?} equals {:?}",
            a, b
        )))
    }
}

pub fn assert_eq_borrow<A: Debug, B: PartialEq<A> + Debug>(a: &A, b: B) -> Result<()> {
    if b.eq(a) {
        Ok(())
    } else {
        Err(Report::msg(format!(
            "Failed asserting that {:?} equals {:?}",
            a, b
        )))
    }
}
