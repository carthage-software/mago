use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;

pub fn get_global_variable_type(variable_name: &str) -> Option<TUnion> {
    Some(match variable_name {
        "$argv" => {
            let mut argv = TUnion::new(vec![
                TAtomic::Null,
                TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![TAtomic::Scalar(
                    TScalar::string(),
                )]))))),
            ]);
            argv.ignore_nullable_issues = true;

            argv
        }
        "$argc" => {
            let mut argc = TUnion::new(vec![TAtomic::Null, TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))]);

            argc.ignore_nullable_issues = true;
            argc
        }
        "$http_response_header" => {
            let mut http_response_header =
                TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::String(TString::general_with_props(false, true, true, false))),
                ])))))]);

            http_response_header.possibly_undefined = true; // undefined in cli
            http_response_header
        }
        "$GLOBALS" => {
            let mut known_items = BTreeMap::new();
            known_items.insert(
                ArrayKey::String("arvc".to_owned()),
                (true, TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))])),
            );
            known_items.insert(
                ArrayKey::String("argv".to_owned()),
                (
                    true,
                    TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                        TAtomic::Scalar(TScalar::string()),
                    ])))))]),
                ),
            );

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_mixed()),
                )),
                non_empty: true,
            }))])
        }
        "$_COOKIE" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::string())])),
        )))]),
        "$_GET" | "$_POST" | "$_REQUEST" => {
            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::non_empty_string()),
                    TAtomic::Scalar(TScalar::int()),
                ])),
                Box::new(TUnion::new(vec![
                    TAtomic::Scalar(TScalar::string()),
                    TAtomic::Array(TArray::Keyed(
                        TKeyedArray::new_with_parameters(
                            Box::new(TUnion::new(vec![
                                TAtomic::Scalar(TScalar::non_empty_string()),
                                TAtomic::Scalar(TScalar::int()),
                            ])),
                            Box::new(TUnion::new(vec![
                                TAtomic::Scalar(TScalar::string()),
                                TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                                    Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::non_empty_string()),
                                        TAtomic::Scalar(TScalar::int()),
                                    ])),
                                    Box::new(get_mixed()),
                                ))),
                            ])),
                        )
                        .to_non_empty(),
                    )),
                ])),
            )))])
        }
        "$_SERVER" => {
            let time =
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

            let mut known_items = BTreeMap::new();
            // Standard CGI/1.1 and PHP variables
            known_items.insert(ArrayKey::String("PHP_SELF".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("GATEWAY_INTERFACE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_ADDR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_SOFTWARE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_PROTOCOL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_METHOD".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_TIME".to_owned()), (true, get_int_range(Some(time), None)));
            known_items.insert(ArrayKey::String("REQUEST_TIME_FLOAT".to_owned()), (true, get_float()));
            known_items.insert(ArrayKey::String("QUERY_STRING".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("DOCUMENT_ROOT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_CHARSET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_ENCODING".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_ACCEPT_LANGUAGE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_REFERER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_USER_AGENT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTPS".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("REMOTE_ADDR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REMOTE_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REMOTE_PORT".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("REMOTE_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REDIRECT_REMOTE_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SCRIPT_FILENAME".to_owned()), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_ADMIN".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SERVER_SIGNATURE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH_TRANSLATED".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SCRIPT_NAME".to_owned()), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_URI".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_DIGEST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PHP_AUTH_PW".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("AUTH_TYPE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH_INFO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("ORIG_PATH_INFO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CONTENT_LENGTH".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("CONTENT_TYPE".to_owned()), (true, get_string()));
            // Common, miscellaneous variables
            known_items.insert(ArrayKey::String("FCGI_ROLE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HOME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CACHE_CONTROL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_COOKIE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_PRIORITY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("PATH".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REDIRECT_STATUS".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("REQUEST_SCHEME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("USER".to_owned()), (true, get_non_empty_string()));
            // Common proxy and CDN headers
            known_items
                .insert(ArrayKey::String("HTTP_UPGRADE_INSECURE_REQUESTS".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_FORWARDED_PROTO".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CLIENT_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_REAL_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_X_FORWARDED_FOR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_CONNECTING_IP".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_IPCOUNTRY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CF_VISITOR".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_CDN_LOOP".to_owned()), (true, get_non_empty_string()));
            // Common Sec-Fetch headers
            known_items.insert(ArrayKey::String("HTTP_DNT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_DEST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_MODE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_FETCH_SITE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA_PLATFORM".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA_MOBILE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HTTP_SEC_CH_UA".to_owned()), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String("APP_DEBUG".to_owned()), (true, get_bool()));
            known_items.insert(ArrayKey::String("APP_ENV".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("APP_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_KEY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_FALLBACK_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DATABASE_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CACHE_DRIVER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_DATABASE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_USERNAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PASSWORD".to_owned()), (true, get_string()));

            known_items.insert(
                ArrayKey::String("arvc".to_owned()),
                (true, TUnion::new(vec![TAtomic::Scalar(TScalar::Integer(TInteger::From(1)))])),
            );
            known_items.insert(
                ArrayKey::String("argv".to_owned()),
                (
                    true,
                    TUnion::new(vec![TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                        TAtomic::Scalar(TScalar::string()),
                    ])))))]),
                ),
            );

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_string()),
                )),
                non_empty: true,
            }))])
        }
        "$_ENV" => {
            let mut known_items = BTreeMap::new();

            // Standard environment variables
            known_items.insert(ArrayKey::String("PATH".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("HOME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("USER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SHELL".to_owned()), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String("APP_DEBUG".to_owned()), (true, get_bool()));
            known_items.insert(ArrayKey::String("APP_ENV".to_owned()), (true, get_string()));
            known_items.insert(ArrayKey::String("APP_NAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_KEY".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("SECRET".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("APP_FALLBACK_LOCALE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DATABASE_URL".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("CACHE_DRIVER".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_CONNECTION".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_HOST".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PORT".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_DATABASE".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_USERNAME".to_owned()), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String("DB_PASSWORD".to_owned()), (true, get_string()));

            TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((
                    Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                    Box::new(get_string()),
                )),
                non_empty: true,
            }))])
        }
        "$_FILES" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: None,
            parameters: Some((
                Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
                Box::new(TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: Some(BTreeMap::from([
                        (
                            ArrayKey::String("name".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("type".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("tmp_name".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("full_path".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("error".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    ]))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String("size".to_owned()),
                            (
                                true,
                                TUnion::new(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::From(0))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::new(vec![
                                        TAtomic::Scalar(TScalar::Integer(TInteger::From(0))),
                                    ]))))),
                                ]),
                            ),
                        ),
                    ])),
                    parameters: None,
                    non_empty: true,
                }))])),
            )),
            non_empty: true,
        }))]),
        "$_SESSION" => TUnion::new(vec![TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(TUnion::new(vec![TAtomic::Scalar(TScalar::non_empty_string())])),
            Box::new(get_mixed()),
        )))]),
        _ => return None,
    })
}
