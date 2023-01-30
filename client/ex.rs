#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use client::{
    register_cli_command, HealthCheckHandler, NodeHandler, register_server_routes,
};
use lazy_static::lazy_static;
#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
#[allow(dead_code)]
struct HEALTH {
    __private_field: (),
}
#[doc(hidden)]
static HEALTH: HEALTH = HEALTH { __private_field: () };
impl ::lazy_static::__Deref for HEALTH {
    type Target = Arc<HealthCheckHandler>;
    fn deref(&self) -> &Arc<HealthCheckHandler> {
        #[inline(always)]
        fn __static_ref_initialize() -> Arc<HealthCheckHandler> {
            Arc::new(HealthCheckHandler {})
        }
        #[inline(always)]
        fn __stability() -> &'static Arc<HealthCheckHandler> {
            static LAZY: ::lazy_static::lazy::Lazy<Arc<HealthCheckHandler>> = ::lazy_static::lazy::Lazy::INIT;
            LAZY.get(__static_ref_initialize)
        }
        __stability()
    }
}
impl ::lazy_static::LazyStatic for HEALTH {
    fn initialize(lazy: &Self) {
        let _ = &**lazy;
    }
}
use std::convert::Infallible;
use client::ClientNodeConfig;
use std::sync::Arc;
use warp::Filter;
use client::RPCNodeHandler;
use paste::paste;
async fn h_e_a_l_t_h() -> Result<impl warp::Reply, Infallible> {
    Ok(warp::reply::json(&HEALTH.handle(&()).await.unwrap()))
}
pub struct IPCClientNode {
    config: ClientNodeConfig,
}
impl IPCClientNode {
    pub fn new(config: ClientNodeConfig) -> Self {
        Self { config }
    }
    /// Runs the node in the current thread
    pub async fn run(&self) {
        let json_rpc = warp::post()
            .and(warp::path("json_rpc"))
            .and(warp::body::bytes())
            .map(|bytes: bytes::Bytes| {
                {
                    let lvl = ::log::Level::Info;
                    if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                        ::log::__private_api_log(
                            ::core::fmt::Arguments::new_v1(
                                &["received bytes = "],
                                &[::core::fmt::ArgumentV1::new_debug(&bytes)],
                            ),
                            lvl,
                            &("ipc", "ipc", "client/src/bin/ipc.rs", 11u32),
                            ::log::__private_api::Option::None,
                        );
                    }
                };
                warp::reply::json(&())
            });
        let routes = json_rpc;
        let routes = routes
            .or(warp::get().and(warp::path("/health-check")).and_then(h_e_a_l_t_h));
        warp::serve(routes).run(self.config.addr()).await;
    }
}
use clap::{Parser, Subcommand};
use client::CommandLineHandler;
/// The overall command line struct
#[command(name = "ipc", about = "The IPC node command line tool", version = "v0.0.1")]
#[command(propagate_version = true)]
struct IPCNode {
    #[command(subcommand)]
    command: Commands,
}
#[automatically_derived]
impl ::core::fmt::Debug for IPCNode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "IPCNode",
            "command",
            &&self.command,
        )
    }
}
impl clap::Parser for IPCNode {}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
)]
#[deny(clippy::correctness)]
impl clap::CommandFactory for IPCNode {
    fn command<'b>() -> clap::Command {
        let __clap_app = clap::Command::new("ipc");
        <Self as clap::Args>::augment_args(__clap_app)
    }
    fn command_for_update<'b>() -> clap::Command {
        let __clap_app = clap::Command::new("ipc");
        <Self as clap::Args>::augment_args_for_update(__clap_app)
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
)]
#[deny(clippy::correctness)]
impl clap::FromArgMatches for IPCNode {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = IPCNode {
            command: {
                <Commands as clap::FromArgMatches>::from_arg_matches_mut(
                    __clap_arg_matches,
                )?
            },
        };
        ::std::result::Result::Ok(v)
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        {
            #[allow(non_snake_case)]
            let command = &mut self.command;
            <Commands as clap::FromArgMatches>::update_from_arg_matches_mut(
                command,
                __clap_arg_matches,
            )?;
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
)]
#[deny(clippy::correctness)]
impl clap::Args for IPCNode {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("IPCNode"))
    }
    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("IPCNode")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 0usize] = [];
                            members
                        }),
                );
            let __clap_app = <Commands as clap::Subcommand>::augment_subcommands(
                __clap_app,
            );
            let __clap_app = __clap_app
                .subcommand_required(true)
                .arg_required_else_help(true);
            __clap_app
                .about("The overall command line struct")
                .long_about(None)
                .about("The IPC node command line tool")
                .version("v0.0.1")
                .propagate_version(true)
        }
    }
    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("IPCNode")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 0usize] = [];
                            members
                        }),
                );
            let __clap_app = <Commands as clap::Subcommand>::augment_subcommands(
                __clap_app,
            );
            let __clap_app = __clap_app
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand_required(false)
                .arg_required_else_help(false);
            __clap_app
                .about("The overall command line struct")
                .long_about(None)
                .about("The IPC node command line tool")
                .version("v0.0.1")
                .propagate_version(true)
        }
    }
}
/// The subcommand to be called
enum Commands {
    HealthCheck(<HealthCheckHandler as CommandLineHandler>::Request),
    Node(<NodeHandler as CommandLineHandler>::Request),
}
#[automatically_derived]
impl ::core::fmt::Debug for Commands {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Commands::HealthCheck(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(
                    f,
                    "HealthCheck",
                    &__self_0,
                )
            }
            Commands::Node(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Node", &__self_0)
            }
        }
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
)]
#[deny(clippy::correctness)]
impl clap::FromArgMatches for Commands {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        if let Some((__clap_name, mut __clap_arg_sub_matches))
            = __clap_arg_matches.remove_subcommand()
        {
            let __clap_arg_matches = &mut __clap_arg_sub_matches;
            if __clap_name == "health-check" && !__clap_arg_matches.contains_id("") {
                return ::std::result::Result::Ok(
                    Self::HealthCheck(
                        <<HealthCheckHandler as CommandLineHandler>::Request as clap::FromArgMatches>::from_arg_matches_mut(
                            __clap_arg_matches,
                        )?,
                    ),
                );
            }
            if __clap_name == "node" && !__clap_arg_matches.contains_id("") {
                return ::std::result::Result::Ok(
                    Self::Node(
                        <<NodeHandler as CommandLineHandler>::Request as clap::FromArgMatches>::from_arg_matches_mut(
                            __clap_arg_matches,
                        )?,
                    ),
                );
            }
            ::std::result::Result::Err(
                clap::Error::raw(
                    clap::error::ErrorKind::InvalidSubcommand,
                    {
                        let res = ::alloc::fmt::format(
                            ::core::fmt::Arguments::new_v1(
                                &["The subcommand \'", "\' wasn\'t recognized"],
                                &[::core::fmt::ArgumentV1::new_display(&__clap_name)],
                            ),
                        );
                        res
                    },
                ),
            )
        } else {
            ::std::result::Result::Err(
                clap::Error::raw(
                    clap::error::ErrorKind::MissingSubcommand,
                    "A subcommand is required but one was not provided.",
                ),
            )
        }
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut<'b>(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        if let Some(__clap_name) = __clap_arg_matches.subcommand_name() {
            match self {
                Self::HealthCheck(
                    ref mut __clap_arg,
                ) if "health-check" == __clap_name => {
                    let (_, mut __clap_arg_sub_matches) = __clap_arg_matches
                        .remove_subcommand()
                        .unwrap();
                    let __clap_arg_matches = &mut __clap_arg_sub_matches;
                    clap::FromArgMatches::update_from_arg_matches_mut(
                        __clap_arg,
                        __clap_arg_matches,
                    )?
                }
                Self::Node(ref mut __clap_arg) if "node" == __clap_name => {
                    let (_, mut __clap_arg_sub_matches) = __clap_arg_matches
                        .remove_subcommand()
                        .unwrap();
                    let __clap_arg_matches = &mut __clap_arg_sub_matches;
                    clap::FromArgMatches::update_from_arg_matches_mut(
                        __clap_arg,
                        __clap_arg_matches,
                    )?
                }
                s => {
                    *s = <Self as clap::FromArgMatches>::from_arg_matches_mut(
                        __clap_arg_matches,
                    )?;
                }
            }
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(dead_code, unreachable_code, unused_variables, unused_braces)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
)]
#[deny(clippy::correctness)]
impl clap::Subcommand for Commands {
    fn augment_subcommands<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = __clap_app
            .subcommand({
                let __clap_subcommand = clap::Command::new("health-check");
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand = {
                    <<HealthCheckHandler as CommandLineHandler>::Request as clap::Args>::augment_args(
                        __clap_subcommand,
                    )
                };
                __clap_subcommand
            });
        let __clap_app = __clap_app
            .subcommand({
                let __clap_subcommand = clap::Command::new("node");
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand = {
                    <<NodeHandler as CommandLineHandler>::Request as clap::Args>::augment_args(
                        __clap_subcommand,
                    )
                };
                __clap_subcommand
            });
        __clap_app.about("The subcommand to be called").long_about(None)
    }
    fn augment_subcommands_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        let __clap_app = __clap_app;
        let __clap_app = __clap_app
            .subcommand({
                let __clap_subcommand = clap::Command::new("health-check");
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand = {
                    <<HealthCheckHandler as CommandLineHandler>::Request as clap::Args>::augment_args_for_update(
                        __clap_subcommand,
                    )
                };
                __clap_subcommand
            });
        let __clap_app = __clap_app
            .subcommand({
                let __clap_subcommand = clap::Command::new("node");
                let __clap_subcommand = __clap_subcommand;
                let __clap_subcommand = {
                    <<NodeHandler as CommandLineHandler>::Request as clap::Args>::augment_args_for_update(
                        __clap_subcommand,
                    )
                };
                __clap_subcommand
            });
        __clap_app.about("The subcommand to be called").long_about(None)
    }
    fn has_subcommand(__clap_name: &str) -> bool {
        if "health-check" == __clap_name {
            return true;
        }
        if "node" == __clap_name {
            return true;
        }
        false
    }
}
pub async fn cli() {
    let args = IPCNode::parse();
    let r = match &args.command {
        Commands::HealthCheck(n) => {
            <HealthCheckHandler as CommandLineHandler>::handle(n).await
        }
        Commands::Node(n) => <NodeHandler as CommandLineHandler>::handle(n).await,
    };
    if r.is_err() {
        {
            ::std::io::_print(
                ::core::fmt::Arguments::new_v1(
                    &["process command: ", " failed due to ", "\n"],
                    &[
                        ::core::fmt::ArgumentV1::new_debug(&args.command),
                        ::core::fmt::ArgumentV1::new_debug(&r.unwrap_err()),
                    ],
                ),
            );
        }
    }
}
fn main() {
    let body = async {
        env_logger::init();
        cli().await;
    };
    #[allow(clippy::expect_used, clippy::diverging_sub_expression)]
    {
        return tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}
