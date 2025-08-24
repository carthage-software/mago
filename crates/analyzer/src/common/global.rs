use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::LazyLock;

use ahash::HashMap;
use mago_atom::atom;
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

std::thread_local! {
    static SUPERGLOBALS_MAP: LazyLock<HashMap<&'static str, Rc<TUnion>>> = LazyLock::new(|| {
        let mut map = HashMap::default();

        map.insert("$argv", Rc::new({
            let mut type_union = TUnion::from_vec(vec![
                TAtomic::Null,
                TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(get_string())))),
            ]);

            type_union.ignore_nullable_issues = true;
            type_union
        }));

        map.insert("$argc", Rc::new({
            let mut type_union = get_one_int();

            type_union.ignore_nullable_issues = true;
            type_union
        }));

        map.insert("$http_response_header", Rc::new({
            let mut type_union =
                TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(get_truthy_string())))));

            type_union.possibly_undefined = true;
            type_union
        }));

        map.insert("$GLOBALS", Rc::new({
            let mut known_items = BTreeMap::new();
            known_items.insert(ArrayKey::String(atom("arvc")), (true, get_positive_int()));
            known_items.insert(
                ArrayKey::String(atom("argv")),
                (true, TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(get_string())))))),
            );

            TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((Box::new(get_non_empty_string()), Box::new(get_mixed()))),
                non_empty: true,
            })))
        }));

        let user_input_type_union = Rc::new( TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(TUnion::from_vec(vec![
                TAtomic::Scalar(TScalar::non_empty_string()),
                TAtomic::Scalar(TScalar::int()),
            ])),
            Box::new(TUnion::from_vec(vec![
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Array(TArray::Keyed(
                    TKeyedArray::new_with_parameters(
                        Box::new(TUnion::from_vec(vec![
                            TAtomic::Scalar(TScalar::non_empty_string()),
                            TAtomic::Scalar(TScalar::int()),
                        ])),
                        Box::new(TUnion::from_vec(vec![
                            TAtomic::Scalar(TScalar::string()),
                            TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                                Box::new(TUnion::from_vec(vec![
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
        )))));

        map.insert("$_GET", user_input_type_union.clone());
        map.insert("$_POST", user_input_type_union.clone());
        map.insert("$_REQUEST", user_input_type_union);

        map.insert("$_SERVER", Rc::new({
            let time = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as i64;

            let mut known_items = BTreeMap::new();
            // Standard CGI/1.1 and PHP variables
            known_items.insert(ArrayKey::String(atom("PHP_SELF")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("GATEWAY_INTERFACE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_ADDR")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_NAME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_SOFTWARE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_PROTOCOL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REQUEST_METHOD")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REQUEST_TIME")), (true, get_int_range(Some(time), None)));
            known_items.insert(ArrayKey::String(atom("REQUEST_TIME_FLOAT")), (true, get_float()));
            known_items.insert(ArrayKey::String(atom("QUERY_STRING")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("DOCUMENT_ROOT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_ACCEPT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_ACCEPT_CHARSET")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_ACCEPT_ENCODING")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_ACCEPT_LANGUAGE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_CONNECTION")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_HOST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_REFERER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_USER_AGENT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTPS")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("REMOTE_ADDR")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REMOTE_HOST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REMOTE_PORT")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("REMOTE_USER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REDIRECT_REMOTE_USER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SCRIPT_FILENAME")), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_ADMIN")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_PORT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SERVER_SIGNATURE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PATH_TRANSLATED")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SCRIPT_NAME")), (false, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REQUEST_URI")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PHP_AUTH_DIGEST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PHP_AUTH_USER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PHP_AUTH_PW")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("AUTH_TYPE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PATH_INFO")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("ORIG_PATH_INFO")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("CONTENT_LENGTH")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("CONTENT_TYPE")), (true, get_string()));
            // Common, miscellaneous variables
            known_items.insert(ArrayKey::String(atom("FCGI_ROLE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HOME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_CACHE_CONTROL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_COOKIE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_PRIORITY")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("PATH")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REDIRECT_STATUS")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("REQUEST_SCHEME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("USER")), (true, get_non_empty_string()));
            // Common proxy and CDN headers
            known_items.insert(
                ArrayKey::String(atom("HTTP_UPGRADE_INSECURE_REQUESTS")),
                (true, get_non_empty_string()),
            );
            known_items
                .insert(ArrayKey::String(atom("HTTP_X_FORWARDED_PROTO")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_CLIENT_IP")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_X_REAL_IP")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_X_FORWARDED_FOR")), (true, get_non_empty_string()));
            known_items
                .insert(ArrayKey::String(atom("HTTP_CF_CONNECTING_IP")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_CF_IPCOUNTRY")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_CF_VISITOR")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_CDN_LOOP")), (true, get_non_empty_string()));
            // Common Sec-Fetch headers
            known_items.insert(ArrayKey::String(atom("HTTP_DNT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_SEC_FETCH_DEST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_SEC_FETCH_USER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_SEC_FETCH_MODE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HTTP_SEC_FETCH_SITE")), (true, get_non_empty_string()));
            known_items
                .insert(ArrayKey::String(atom("HTTP_SEC_CH_UA_PLATFORM")), (true, get_non_empty_string()));
            known_items
                .insert(ArrayKey::String(atom("HTTP_SEC_CH_UA_MOBILE")), (true, get_non_empty_string()));
                known_items.insert(ArrayKey::String(atom("HTTP_SEC_CH_UA")), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String(atom("APP_DEBUG")), (true, get_bool()));
            known_items.insert(ArrayKey::String(atom("APP_ENV")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("APP_NAME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_URL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_KEY")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_SECRET")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SECRET")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_LOCALE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_FALLBACK_LOCALE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DATABASE_URL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("CACHE_DRIVER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_CONNECTION")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_HOST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_PORT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_DATABASE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_USERNAME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_PASSWORD")), (true, get_string()));

            known_items.insert(ArrayKey::String(atom("arvc")), (true, get_positive_int()));
            known_items.insert(
                ArrayKey::String(atom("argv")),
                (true, TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(get_string())))))),
            );

            TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((Box::new(get_non_empty_string()), Box::new(get_string()))),
                non_empty: true,
            })))
        }));

        map.insert("$_ENV", Rc::new({
            let mut known_items = BTreeMap::new();

            // Standard environment variables
            known_items.insert(ArrayKey::String(atom("PATH")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("HOME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("USER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SHELL")), (true, get_non_empty_string()));

            // Common framework and application variables (e.g., Symfony, PHPUnit, Laravel)
            known_items.insert(ArrayKey::String(atom("APP_DEBUG")), (true, get_bool()));
            known_items.insert(ArrayKey::String(atom("APP_ENV")), (true, get_string()));
            known_items.insert(ArrayKey::String(atom("APP_NAME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_URL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_KEY")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_SECRET")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("SECRET")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_LOCALE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("APP_FALLBACK_LOCALE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DATABASE_URL")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("CACHE_DRIVER")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_CONNECTION")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_HOST")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_PORT")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_DATABASE")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_USERNAME")), (true, get_non_empty_string()));
            known_items.insert(ArrayKey::String(atom("DB_PASSWORD")), (true, get_string()));

            TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                known_items: Some(known_items),
                parameters: Some((Box::new(get_non_empty_string()), Box::new(get_string()))),
                non_empty: true,
            })))
        }));

        map.insert("$_FILES", Rc::new(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
            known_items: None,
            parameters: Some((
                Box::new(get_non_empty_string()),
                Box::new(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray {
                    known_items: Some(BTreeMap::from([
                        (
                            ArrayKey::String(atom("name")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(
                                        Box::new(get_non_empty_string()),
                                    ))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String(atom("type")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(
                                        Box::new(get_non_empty_string()),
                                    ))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String(atom("tmp_name")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(
                                        Box::new(get_non_empty_string()),
                                    ))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String(atom("full_path")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::String(TString::non_empty())),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(
                                        Box::new(get_non_empty_string()),
                                    ))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String(atom("error")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(Box::new(TUnion::from_atomic(
                                        TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 8))),
                                    ))))),
                                ]),
                            ),
                        ),
                        (
                            ArrayKey::String(atom("size")),
                            (
                                true,
                                TUnion::from_vec(vec![
                                    TAtomic::Scalar(TScalar::Integer(TInteger::From(0))),
                                    TAtomic::Array(TArray::List(TList::new_non_empty(
                                        Box::new(get_non_negative_int()),
                                    ))),
                                ]),
                            ),
                        ),
                    ])),
                    parameters: None,
                    non_empty: true,
                })))),
            )),
            non_empty: true,
        })))));

        map.insert("$_SESSION", Rc::new(TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
            Box::new(get_non_empty_string()),
            Box::new(get_mixed()),
        ))))));

        map
    });
}

pub fn get_global_variable_type(variable_name: &str) -> Option<Rc<TUnion>> {
    SUPERGLOBALS_MAP.with(|map| map.get(variable_name).cloned())
}
