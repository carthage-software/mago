use ahash::HashMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackage {
    /// Indicates whether this package has been abandoned, it can be boolean or a package name/URL pointing to a recommended alternative.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abandoned: Option<ComposerPackageAbandoned>,

    /// Options for creating package archives for distribution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<ComposerPackageArchive>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Authors>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoload: Option<Autoload>,

    #[serde(rename = "autoload-dev", default, skip_serializing_if = "Option::is_none")]
    pub autoload_dev: Option<ComposerPackageAutoloadDev>,

    /// A set of files, or a single file, that should be treated as binaries and symlinked into bin-dir (from config)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<ComposerPackageBin>,

    /// A key to store comments in"]
    #[serde(rename = "_comment", default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<ComposerPackageComment>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<ComposerPackageConfig>,

    /// This is an object of package name (keys) and version constraints (values) that conflict with this package
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub conflict: HashMap<String, String>,

    /// Internal use only, do not specify this in composer.json. Indicates whether this version is the default branch of the linked VCS repository. Defaults to false
    #[serde(rename = "default-branch", default, skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<bool>,

    /// Short package description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dist: Option<Dist>,

    /// Arbitrary extra data that can be used by plugins, for example, package of type composer-plugin may have a 'class' key defining an installer class name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<ComposerPackageExtra>,

    /// A list of options to fund the development and maintenance of the package
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub funding: Vec<ComposerPackageFundingItem>,

    /// Homepage URL for the project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// A list of directories which should get added to PHP's include path.
    #[serde(rename = "include-path", default, skip_serializing_if = "Vec::is_empty")]
    #[deprecated(
        note = "This is only present to support legacy projects, and all new code should preferably use autoloading"
    )]
    pub include_path: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// License name. Or an array of license names
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<ComposerPackageLicense>,

    /// The minimum stability the packages must have to be install-able. Possible values are: dev, alpha, beta, RC, stable
    #[serde(rename = "minimum-stability", default, skip_serializing_if = "Option::is_none")]
    pub minimum_stability: Option<ComposerPackageMinimumStability>,

    /// Package name, including 'vendor-name/' prefix
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<ComposerPackageName>,

    /// A set of string or regex patterns for non-numeric branch names that will not be handled as feature branches
    #[serde(rename = "non-feature-branches", default, skip_serializing_if = "Vec::is_empty")]
    pub non_feature_branches: Vec<String>,

    #[serde(rename = "php-ext", default, skip_serializing_if = "Option::is_none")]
    pub php_ext: Option<ComposerPackagePhpExt>,

    /// If set to true, stable packages will be preferred to dev packages when possible, even if the minimum-stability allows unstable packages
    #[serde(rename = "prefer-stable", default, skip_serializing_if = "Option::is_none")]
    pub prefer_stable: Option<bool>,

    /// This is an object of package name (keys) and version constraints (values) that this package provides in addition to this package's name
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub provide: HashMap<String, String>,

    /// Relative path to the readme document
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readme: Option<String>,

    /// This is an object of package name (keys) and version constraints (values) that can be replaced by this package
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub replace: HashMap<String, String>,

    /// A set of additional repositories where packages can be found
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repositories: Option<ComposerPackageRepositories>,

    /// This is an object of package name (keys) and version constraints (values) that are required to run this package
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub require: HashMap<String, String>,

    /// This is an object of package name (keys) and version constraints (values) that this package requires for developing it (testing tools and such)
    #[serde(rename = "require-dev", default, skip_serializing_if = "HashMap::is_empty")]
    pub require_dev: HashMap<String, String>,

    /// Script listeners that will be executed before/after some events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scripts: Option<ComposerPackageScripts>,

    /// Aliases for custom commands
    #[serde(rename = "scripts-aliases", default, skip_serializing_if = "HashMap::is_empty")]
    pub scripts_aliases: HashMap<String, Vec<String>>,

    /// Descriptions for custom commands, shown in console help
    #[serde(rename = "scripts-descriptions", default, skip_serializing_if = "HashMap::is_empty")]
    pub scripts_descriptions: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    /// This is an object of package name (keys) and descriptions (values) that this package suggests work well with it (this will be suggested to the user during installation)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub suggest: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support: Option<ComposerPackageSupport>,

    /// Forces the package to be installed into the given subdirectory path. This is used for autoloading PSR-0 packages that do not contain their full path.
    /// Use forward slashes for cross-platform compatibility
    #[serde(rename = "target-dir", default, skip_serializing_if = "Option::is_none")]
    #[deprecated]
    pub target_dir: Option<String>,

    /// Package release date, in 'YYYY-MM-DD', 'YYYY-MM-DD HH:MM:SS' or 'YYYY-MM-DDTHH:MM:SSZ' format
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    /// Package type, either 'library' for common packages, 'composer-plugin' for plugins, 'metapackage' for empty packages, or a custom type ([a-z0-9-]+) defined by whatever project this package applies to
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ComposerPackageType>,

    /// Package version, see https://getcomposer.org/doc/04-schema.md#version for more info on valid schemes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<ComposerPackageVersion>,
}

/// Indicates whether this package has been abandoned, it can be boolean or a package name/URL pointing to a recommended alternative.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageAbandoned {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageArchive {
    /// A list of patterns for paths to exclude or include if prefixed with an exclamation mark
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    /// A base name for archive
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageAutoloadDev {
    /// This is an array of paths that contain classes to be included in the class-map generation process
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classmap: Vec<String>,

    /// This is an array of files that are always required on every request
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,

    /// This is an object of namespaces (keys) and the directories they can be found into (values, can be arrays of paths) by the autoloader
    #[serde(rename = "psr-0", default, skip_serializing_if = "HashMap::is_empty")]
    pub psr_0: HashMap<String, ComposerPackageAutoloadDevPsr0value>,

    /// This is an object of namespaces (keys) and the PSR-4 directories they can map to (values, can be arrays of paths) by the autoloader
    #[serde(rename = "psr-4", default, skip_serializing_if = "HashMap::is_empty")]
    pub psr_4: HashMap<String, ComposerPackageAutoloadDevPsr4value>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageAutoloadDevPsr0value {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageAutoloadDevPsr4value {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageBin {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageComment {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageConfig {
    /// Defaults to false. If set to true, Composer will allow install when lock file is not up to date with the latest changes in composer.json
    #[serde(rename = "allow-missing-requirements", default, skip_serializing_if = "Option::is_none")]
    pub allow_missing_requirements: Option<bool>,

    /// This is an object of {"pattern": true|false} with packages which are allowed to be loaded as plugins, or true to allow all, false to allow none. Defaults to {} which prompts when an unknown plugin is added
    #[serde(rename = "allow-plugins", default, skip_serializing_if = "Option::is_none")]
    pub allow_plugins: Option<ComposerPackageConfigAllowPlugins>,

    /// If true, the Composer autoloader will check for APCu and use it to cache found/not-found classes when the extension is enabled, defaults to false
    #[serde(rename = "apcu-autoloader", default, skip_serializing_if = "Option::is_none")]
    pub apcu_autoloader: Option<bool>,

    /// The default archive path when not provided on cli, defaults to "."
    #[serde(rename = "archive-dir", default, skip_serializing_if = "Option::is_none")]
    pub archive_dir: Option<String>,

    /// The default archiving format when not provided on cli, defaults to "tar"
    #[serde(rename = "archive-format", default, skip_serializing_if = "Option::is_none")]
    pub archive_format: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<ComposerPackageConfigAudit>,

    /// Optional string to be used as a suffix for the generated Composer autoloader. When null a random one will be generated
    #[serde(rename = "autoloader-suffix", default, skip_serializing_if = "Option::is_none")]
    pub autoloader_suffix: Option<String>,

    /// An object of domain name => bearer authentication token, for example {"example.com":"<token>"}
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bearer: HashMap<String, String>,

    /// The compatibility of the binaries, defaults to "auto" (automatically guessed), can be "full" (compatible with both Windows and Unix-based systems) and "proxy" (only bash-style proxy)
    #[serde(rename = "bin-compat", default, skip_serializing_if = "Option::is_none")]
    pub bin_compat: Option<ComposerPackageConfigBinCompat>,

    /// The location where all binaries are linked, defaults to "vendor/bin"
    #[serde(rename = "bin-dir", default, skip_serializing_if = "Option::is_none")]
    pub bin_dir: Option<String>,

    /// An object of domain name => {"consumer-key": "...", "consumer-secret": "..."}
    #[serde(rename = "bitbucket-oauth", default, skip_serializing_if = "HashMap::is_empty")]
    pub bitbucket_oauth: HashMap<String, ComposerPackageConfigBitbucketOauthValue>,

    /// Defaults to false and can be any of true, false, "dev"` or "no-dev"`. If set to true, Composer will run the bump command after running the update command. If set to "dev" or "no-dev" then only the corresponding dependencies will be bumped
    #[serde(rename = "bump-after-update", default, skip_serializing_if = "Option::is_none")]
    pub bump_after_update: Option<ComposerPackageConfigBumpAfterUpdate>,

    /// The location where all caches are located, defaults to "~/.composer/cache" on *nix and "%LOCALAPPDATA%\\Composer" on windows
    #[serde(rename = "cache-dir", default, skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,

    /// The location where files (zip downloads) are cached, defaults to "{$cache-dir}/files"
    #[serde(rename = "cache-files-dir", default, skip_serializing_if = "Option::is_none")]
    pub cache_files_dir: Option<String>,

    /// The cache max size for the files cache, defaults to "300MiB"
    #[serde(rename = "cache-files-maxsize", default, skip_serializing_if = "Option::is_none")]
    pub cache_files_maxsize: Option<ComposerPackageConfigCacheFilesMaxsize>,

    /// The cache time-to-live for files, defaults to the value of cache-ttl
    #[serde(rename = "cache-files-ttl", default, skip_serializing_if = "Option::is_none")]
    pub cache_files_ttl: Option<usize>,

    /// Whether to use the Composer cache in read-only mode
    #[serde(rename = "cache-read-only", default, skip_serializing_if = "Option::is_none")]
    pub cache_read_only: Option<bool>,

    /// The location where repo (git/hg repo clones) are cached, defaults to "{$cache-dir}/repo"
    #[serde(rename = "cache-repo-dir", default, skip_serializing_if = "Option::is_none")]
    pub cache_repo_dir: Option<String>,

    /// The default cache time-to-live, defaults to 15552000 (6 months)
    #[serde(rename = "cache-ttl", default, skip_serializing_if = "Option::is_none")]
    pub cache_ttl: Option<usize>,

    /// The location where vcs infos (git clones, github api calls, etc. when reading vcs repos) are cached, defaults to "{$cache-dir}/vcs"
    #[serde(rename = "cache-vcs-dir", default, skip_serializing_if = "Option::is_none")]
    pub cache_vcs_dir: Option<String>,

    /// A way to set the path to the openssl CA file. In PHP 5.6+ you should rather set this via openssl.cafile in php.ini, although PHP 5.6+ should be able to detect your system CA file automatically
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cafile: Option<String>,

    /// If cafile is not specified or if the certificate is not found there, the directory pointed to by capath is searched for a suitable certificate. capath must be a correctly hashed certificate directory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capath: Option<String>,

    /// If true, the composer autoloader will not scan the filesystem for classes that are not found in the class map, defaults to false
    #[serde(rename = "classmap-authoritative", default, skip_serializing_if = "Option::is_none")]
    pub classmap_authoritative: Option<bool>,

    /// The location where old phar files are stored, defaults to "$home" except on XDG Base Directory compliant unixes
    #[serde(rename = "data-dir", default, skip_serializing_if = "Option::is_none")]
    pub data_dir: Option<String>,

    /// Defaults to `false`. If set to true all HTTPS URLs will be tried with HTTP instead and no network level encryption is performed. Enabling this is a security risk and is NOT recommended. The better way is to enable the php_openssl extension in php.ini
    #[serde(rename = "disable-tls", default, skip_serializing_if = "Option::is_none")]
    pub disable_tls: Option<bool>,

    /// The default style of handling dirty updates, defaults to false and can be any of true, false or "stash"
    #[serde(rename = "discard-changes", default, skip_serializing_if = "Option::is_none")]
    pub discard_changes: Option<ComposerPackageConfigDiscardChanges>,

    /// A list of domains to use in github mode. This is used for GitHub Enterprise setups, defaults to ["github.com"]
    #[serde(rename = "github-domains", default, skip_serializing_if = "Vec::is_empty")]
    pub github_domains: Vec<String>,

    /// Defaults to true. If set to false, the OAuth tokens created to access the github API will have a date instead of the machine hostname
    #[serde(rename = "github-expose-hostname", default, skip_serializing_if = "Option::is_none")]
    pub github_expose_hostname: Option<bool>,

    /// An object of domain name => github API oauth tokens, typically {"github.com":"<token>"}
    #[serde(rename = "github-oauth", default, skip_serializing_if = "HashMap::is_empty")]
    pub github_oauth: HashMap<String, String>,

    /// A list of protocols to use for github.com clones, in priority order, defaults to ["https", "ssh", "git"]
    #[serde(rename = "github-protocols", default, skip_serializing_if = "Vec::is_empty")]
    pub github_protocols: Vec<String>,

    /// A list of domains to use in gitlab mode. This is used for custom GitLab setups, defaults to ["gitlab.com"]
    #[serde(rename = "gitlab-domains", default, skip_serializing_if = "Vec::is_empty")]
    pub gitlab_domains: Vec<String>,

    /// An object of domain name => gitlab API oauth tokens, typically {"gitlab.com":{"expires-at":"<expiration date>", "refresh-token":"<refresh token>", "token":"<token>"}}
    #[serde(rename = "gitlab-oauth", default, skip_serializing_if = "HashMap::is_empty")]
    pub gitlab_oauth: HashMap<String, ComposerPackageConfigGitlabOauthValue>,

    /// A protocol to force use of when creating a repository URL for the `source` value of the package metadata. One of `git` or `http`. By default, Composer will generate a git URL for private repositories and http one for public repos
    #[serde(rename = "gitlab-protocol", default, skip_serializing_if = "Option::is_none")]
    pub gitlab_protocol: Option<ComposerPackageConfigGitlabProtocol>,

    /// An object of domain name => gitlab private tokens, typically {"gitlab.com":"<token>"}, or an object with username and token keys
    #[serde(rename = "gitlab-token", default, skip_serializing_if = "HashMap::is_empty")]
    pub gitlab_token: HashMap<String, ComposerPackageConfigGitlabTokenValue>,

    /// Defaults to true. If set to false, Composer will not create .htaccess files in the composer home, cache, and data directories
    #[serde(rename = "htaccess-protect", default, skip_serializing_if = "Option::is_none")]
    pub htaccess_protect: Option<bool>,

    /// An object of domain name => {"username": "...", "password": "..."}
    #[serde(rename = "http-basic", default, skip_serializing_if = "HashMap::is_empty")]
    pub http_basic: HashMap<String, ComposerPackageConfigHttpBasicValue>,

    /// Defaults to true. If set to false, Composer will not create a composer.lock file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lock: Option<bool>,

    /// Composer allows repositories to define a notification URL, so that they get notified whenever a package from that repository is installed. This option allows you to disable that behaviour, defaults to true
    #[serde(rename = "notify-on-install", default, skip_serializing_if = "Option::is_none")]
    pub notify_on_install: Option<bool>,

    /// Always optimize when dumping the autoloader
    #[serde(rename = "optimize-autoloader", default, skip_serializing_if = "Option::is_none")]
    pub optimize_autoloader: Option<bool>,

    /// This is an object of package name (keys) and version (values) that will be used to mock the platform packages on this machine, the version can be set to false to make it appear like the package is not present
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub platform: HashMap<String, ComposerPackageConfigPlatformValue>,

    /// Defaults to "php-only" which checks only the PHP version. Setting to true will also check the presence of required PHP extensions. If set to false, Composer will not create and require a platform_check.php file as part of the autoloader bootstrap
    #[serde(rename = "platform-check", default, skip_serializing_if = "Option::is_none")]
    pub platform_check: Option<ComposerPackageConfigPlatformCheck>,

    /// The install method Composer will prefer to use, defaults to auto and can be any of source, dist, auto, or an object of {"pattern": "preference"}
    #[serde(rename = "preferred-install", default, skip_serializing_if = "Option::is_none")]
    pub preferred_install: Option<ComposerPackageConfigPreferredInstall>,

    /// If false, the composer autoloader will not be prepended to existing autoloaders, defaults to true
    #[serde(rename = "prepend-autoloader", default, skip_serializing_if = "Option::is_none")]
    pub prepend_autoloader: Option<bool>,

    /// The timeout in seconds for process executions, defaults to 300 (5mins)
    #[serde(rename = "process-timeout", default, skip_serializing_if = "Option::is_none")]
    pub process_timeout: Option<usize>,

    /// Defaults to `true`. If set to true only HTTPS URLs are allowed to be downloaded via Composer. If you really absolutely need HTTP access to something then you can disable it, but using "Let's Encrypt" to get a free SSL certificate is generally a better alternative
    #[serde(rename = "secure-http", default, skip_serializing_if = "Option::is_none")]
    pub secure_http: Option<bool>,

    /// A list of domains which should be trusted/marked as using a secure Subversion/SVN transport. By default svn:// protocol is seen as insecure and will throw. This is a better/safer alternative to disabling `secure-http` altogether
    #[serde(rename = "secure-svn-domains", default, skip_serializing_if = "Vec::is_empty")]
    pub secure_svn_domains: Vec<String>,

    /// Defaults to false. If set to true, Composer will sort packages when adding/updating a new dependency
    #[serde(rename = "sort-packages", default, skip_serializing_if = "Option::is_none")]
    pub sort_packages: Option<bool>,

    /// What to do after prompting for authentication, one of: true (store), false (do not store) or "prompt" (ask every time), defaults to prompt
    #[serde(rename = "store-auths", default, skip_serializing_if = "Option::is_none")]
    pub store_auths: Option<ComposerPackageConfigStoreAuths>,

    /// Defaults to true.  If set to false, globally disables the use of the GitHub API for all GitHub repositories and clones the repository as it would for any other repository
    #[serde(rename = "use-github-api", default, skip_serializing_if = "Option::is_none")]
    pub use_github_api: Option<bool>,

    /// If true, the Composer autoloader will also look for classes in the PHP include path
    #[serde(rename = "use-include-path", default, skip_serializing_if = "Option::is_none")]
    pub use_include_path: Option<bool>,

    /// When running Composer in a directory where there is no composer.json, if there is one present in a directory above Composer will by default ask you whether you want to use that directory's composer.json instead. One of: true (always use parent if needed), false (never ask or use it) or "prompt" (ask every time), defaults to prompt
    #[serde(rename = "use-parent-dir", default, skip_serializing_if = "Option::is_none")]
    pub use_parent_dir: Option<ComposerPackageConfigUseParentDir>,

    /// The location where all packages are installed, defaults to "vendor"
    #[serde(rename = "vendor-dir", default, skip_serializing_if = "Option::is_none")]
    pub vendor_dir: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigAllowPlugins {
    Boolean(bool),
    Object(HashMap<String, bool>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageConfigAudit {
    /// Whether abandoned packages should be ignored, reported as problems or cause an audit failure
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abandoned: Option<ComposerPackageConfigAuditAbandoned>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ignore: Option<ComposerPackageConfigAuditIgnore>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposerPackageConfigAuditAbandoned {
    #[serde(rename = "ignore")]
    Ignore,
    #[serde(rename = "report")]
    Report,
    #[serde(rename = "fail")]
    Fail,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigAuditIgnore {
    DescribedIdentifiers(HashMap<String, String>),
    Identifiers(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposerPackageConfigBinCompat {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "full")]
    Full,
    #[serde(rename = "proxy")]
    Proxy,
    #[serde(rename = "symlink")]
    Symlink,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageConfigBitbucketOauthValue {
    /// The OAuth token retrieved from Bitbucket's API, this is written by Composer and you should not set it nor modify it
    #[serde(rename = "access-token", default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,

    /// The generated token's expiration timestamp, this is written by Composer and you should not set it nor modify it
    #[serde(rename = "access-token-expiration", default, skip_serializing_if = "Option::is_none")]
    pub access_token_expiration: Option<isize>,

    /// The consumer-key used for OAuth authentication"]
    #[serde(rename = "consumer-key")]
    pub consumer_key: String,

    /// The consumer-secret used for OAuth authentication"]
    #[serde(rename = "consumer-secret")]
    pub consumer_secret: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigBumpAfterUpdate {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigCacheFilesMaxsize {
    String(String),
    Integer(usize),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigDiscardChanges {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigGitlabOauthValue {
    Object {
        /// The expiration date for this GitLab token
        #[serde(rename = "expires-at", default, skip_serializing_if = "Option::is_none")]
        expires_at: Option<usize>,

        /// The refresh token used for GitLab authentication
        #[serde(rename = "refresh-token", default, skip_serializing_if = "Option::is_none")]
        refresh_token: Option<String>,

        /// The token used for GitLab authentication
        token: String,
    },
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposerPackageConfigGitlabProtocol {
    #[serde(rename = "git")]
    Git,
    #[serde(rename = "http")]
    Http,
    #[serde(rename = "https")]
    Https,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigGitlabTokenValue {
    Object {
        /// The token used for GitLab authentication"]
        token: String,

        /// The username used for GitLab authentication"]
        username: String,
    },
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageConfigHttpBasicValue {
    /// The password used for HTTP Basic authentication"]
    pub password: String,

    /// The username used for HTTP Basic authentication"]
    pub username: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigPlatformCheck {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigPlatformValue {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigPreferredInstall {
    Object(HashMap<String, String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigStoreAuths {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageConfigUseParentDir {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageExtra {
    Object(HashMap<String, Value>),
    Array(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageFundingItem {
    /// Type of funding or platform through which funding is possible
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// URL to a website with details on funding and a way to fund the package
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageLicense {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposerPackageMinimumStability {
    #[serde(rename = "dev")]
    Dev,
    #[serde(rename = "alpha")]
    Alpha,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "rc", alias = "RC")]
    Rc,
    #[serde(rename = "stable")]
    Stable,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComposerPackageName(pub String);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackagePhpExt {
    /// These configure options make up the flags that can be passed to ./configure when installing the extension
    #[serde(rename = "configure-options", default, skip_serializing_if = "Vec::is_empty")]
    pub configure_options: Vec<ComposerPackagePhpExtConfigureOptionsItem>,

    /// If specified, this will be used as the name of the extension, where needed by tooling. If this is not specified, the extension name will be derived from the Composer package name (e.g. `vendor/name` would become `ext-name`). The extension name may be specified with or without the `ext-` prefix, and tools that use this must normalise this appropriately
    #[serde(rename = "extension-name", default, skip_serializing_if = "Option::is_none")]
    pub extension_name: Option<String>,

    /// This is used to add a prefix to the INI file, e.g. `90-xdebug.ini` which affects the loading order. The priority is a number in the range 10-99 inclusive, with 10 being the highest priority (i.e. will be processed first), and 99 being the lowest priority (i.e. will be processed last). There are two digits so that the files sort correctly on any platform, whether the sorting is natural or not
    #[serde(default = "default_composer_package_php_ext_priority")]
    pub priority: usize,

    /// Does this package support non-Thread Safe mode"]
    #[serde(rename = "support-nts", default = "default_composer_package_php_ext_support_nts")]
    pub support_nts: bool,

    /// Does this package support Zend Thread Safety"]
    #[serde(rename = "support-zts", default = "default_composer_package_php_ext_support_zts")]
    pub support_zts: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackagePhpExtConfigureOptionsItem {
    /// The description of what the flag does or means
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The name of the flag, this would typically be prefixed with `--`, for example, the value 'the-flag' would be passed as `./configure --the-flag`
    pub name: ComposerPackagePhpExtConfigureOptionsItemName,

    /// If this is set to true, the flag needs a value (e.g. --with-somelib=<path>), otherwise it is a flag without a value (e.g. --enable-some-feature)
    #[serde(rename = "needs-value", default)]
    pub needs_value: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComposerPackagePhpExtConfigureOptionsItemName(pub String);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageRepositories {
    Object(HashMap<String, ComposerPackageRepositoriesObjectValue>),
    Array(Vec<ComposerPackageRepositoriesArrayItem>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageRepositoriesArrayItem {
    Repository(Box<Repository>),
    Map(HashMap<String, bool>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageRepositoriesObjectValue {
    Repository(Box<Repository>),
    Enabled(bool),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageScripts {
    /// Occurs after the autoloader is dumped, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-autoload-dump", default, skip_serializing_if = "Option::is_none")]
    pub post_autoload_dump: Option<ComposerPackageScriptsCallback>,

    /// Occurs after the create-project command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-create-project-cmd", default, skip_serializing_if = "Option::is_none")]
    pub post_create_project_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs after the install command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-install-cmd", default, skip_serializing_if = "Option::is_none")]
    pub post_install_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs after a package is installed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-package-install", default, skip_serializing_if = "Option::is_none")]
    pub post_package_install: Option<ComposerPackageScriptsCallback>,

    /// Occurs after a package has been uninstalled, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-package-uninstall", default, skip_serializing_if = "Option::is_none")]
    pub post_package_uninstall: Option<ComposerPackageScriptsCallback>,

    /// Occurs after a package is updated, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-package-update", default, skip_serializing_if = "Option::is_none")]
    pub post_package_update: Option<ComposerPackageScriptsCallback>,

    /// Occurs after the root-package is installed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-root-package-install", default, skip_serializing_if = "Option::is_none")]
    pub post_root_package_install: Option<ComposerPackageScriptsCallback>,

    /// Occurs after the status command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-status-cmd", default, skip_serializing_if = "Option::is_none")]
    pub post_status_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs after the update command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "post-update-cmd", default, skip_serializing_if = "Option::is_none")]
    pub post_update_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs before the autoloader is dumped, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-autoload-dump", default, skip_serializing_if = "Option::is_none")]
    pub pre_autoload_dump: Option<ComposerPackageScriptsCallback>,

    /// Occurs before the install command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-install-cmd", default, skip_serializing_if = "Option::is_none")]
    pub pre_install_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs before a package is installed, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-package-install", default, skip_serializing_if = "Option::is_none")]
    pub pre_package_install: Option<ComposerPackageScriptsCallback>,

    /// Occurs before a package has been uninstalled, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-package-uninstall", default, skip_serializing_if = "Option::is_none")]
    pub pre_package_uninstall: Option<ComposerPackageScriptsCallback>,

    /// Occurs before a package is updated, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-package-update", default, skip_serializing_if = "Option::is_none")]
    pub pre_package_update: Option<ComposerPackageScriptsCallback>,

    /// Occurs before the status command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-status-cmd", default, skip_serializing_if = "Option::is_none")]
    pub pre_status_cmd: Option<ComposerPackageScriptsCallback>,

    /// Occurs before the update command is executed, contains one or more Class::method callables or shell commands
    #[serde(rename = "pre-update-cmd", default, skip_serializing_if = "Option::is_none")]
    pub pre_update_cmd: Option<ComposerPackageScriptsCallback>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComposerPackageScriptsCallback {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerPackageSupport {
    /// URL to the support chat
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat: Option<String>,

    /// URL to the documentation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,

    /// Email address for support
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// URL to the forum
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub forum: Option<String>,

    /// IRC channel for support, as irc://server/channel
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub irc: Option<String>,

    /// URL to the issue tracker
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,

    /// URL to the RSS feed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rss: Option<String>,

    /// URL to the vulnerability disclosure policy (VDP)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,

    /// URL to browse or download the sources
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// URL to the wiki
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wiki: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComposerPackageType(pub String);

#[derive(Serialize, Deserialize, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComposerPackageVersion(pub String);

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ArtifactRepositoryType {
    #[serde(rename = "artifact")]
    Artifact,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ArtifactRepository {
    pub r#type: ArtifactRepositoryType,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Authors(pub Vec<AuthorsItem>);

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AuthorsItem {
    /// Email address of the author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Homepage URL for the author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// Full name of the author
    pub name: String,

    /// Author's role in the project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Autoload {
    /// This is an array of paths that contain classes to be included in the class-map generation process
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub classmap: Vec<String>,

    /// This is an array of patterns to exclude from autoload classmap generation. (e.g. "exclude-from-classmap": ["/test/", "/tests/", "/Tests/"]"]
    #[serde(rename = "exclude-from-classmap", default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_from_classmap: Vec<String>,

    /// This is an array of files that are always required on every request
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,

    /// This is an object of namespaces (keys) and the directories they can be found in (values, can be arrays of paths) by the autoloader
    #[serde(rename = "psr-0", default, skip_serializing_if = "HashMap::is_empty")]
    pub psr_0: HashMap<String, AutoloadPsr0value>,

    /// This is an object of namespaces (keys) and the PSR-4 directories they can map to (values, can be arrays of paths) by the autoloader
    #[serde(rename = "psr-4", default, skip_serializing_if = "HashMap::is_empty")]
    pub psr_4: HashMap<String, AutoloadPsr4value>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum AutoloadPsr0value {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum AutoloadPsr4value {
    Array(Vec<String>),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ComposerRepository {
    pub r#type: ComposerRepositoryType,

    pub url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_ssl_downgrade: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    #[serde(rename = "force-lazy-providers", default, skip_serializing_if = "Option::is_none")]
    pub force_lazy_providers: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub options: HashMap<String, Value>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ComposerRepositoryType {
    #[serde(rename = "composer")]
    Composer,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Dist {
    pub r#type: String,

    pub url: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shasum: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct InlinePackage {
    /// Package name, including 'vendor-name/' prefix
    pub name: String,

    pub version: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<InlinePackageArchive>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authors: Option<Authors>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoload: Option<Autoload>,

    /// A set of files, or a single file, that should be treated as binaries and symlinked into bin-dir (from config)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bin: Option<InlinePackageBin>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub conflict: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dist: Option<Dist>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<InlinePackageExtra>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,

    /// A list of directories which should get added to PHP's include path.
    #[serde(rename = "include-path", default, skip_serializing_if = "Vec::is_empty")]
    #[deprecated(
        note = "this is only present to support legacy projects, and all new code should preferably use autoloading"
    )]
    pub include_path: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<InlinePackageLicense>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub provide: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub replace: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub require: HashMap<String, String>,

    #[serde(rename = "require-dev", default, skip_serializing_if = "HashMap::is_empty")]
    pub require_dev: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub suggest: HashMap<String, String>,

    #[serde(rename = "target-dir", default, skip_serializing_if = "Option::is_none")]
    #[deprecated]
    pub target_dir: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct InlinePackageArchive {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum InlinePackageBin {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum InlinePackageExtra {
    Object(HashMap<String, Value>),
    Array(Vec<Value>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum InlinePackageLicense {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PackageRepository {
    pub r#type: PackageRepositoryType,

    pub package: PackageRepositoryPackage,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum PackageRepositoryPackage {
    One(InlinePackage),
    Many(Vec<InlinePackage>),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PackageRepositoryType {
    #[serde(rename = "package")]
    Package,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PathRepository {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<PathRepositoryOptions>,
    #[serde(rename = "type")]
    pub type_: PathRepositoryType,
    pub url: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PathRepositoryOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symlink: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PathRepositoryType {
    #[serde(rename = "path")]
    Path,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PearRepository {
    pub r#type: PearRepositoryType,
    pub url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,

    #[serde(rename = "vendor-alias", default, skip_serializing_if = "Option::is_none")]
    pub vendor_alias: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PearRepositoryType {
    #[serde(rename = "pear")]
    Pear,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum Repository {
    Composer(ComposerRepository),
    Vcs(VcsRepository),
    Path(PathRepository),
    Artifact(ArtifactRepository),
    Pear(PearRepository),
    Package(PackageRepository),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Source {
    pub r#type: String,

    pub url: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mirrors: Vec<Value>,

    pub reference: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct VcsRepository {
    pub r#type: VcsRepositoryType,
    pub url: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    #[serde(rename = "branches-path", default, skip_serializing_if = "Option::is_none")]
    pub branches_path: Option<VcsRepositoryBranchesPath>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical: Option<bool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depot: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude: Vec<String>,

    #[serde(rename = "no-api", default, skip_serializing_if = "Option::is_none")]
    pub no_api: Option<bool>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub only: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p4password: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p4user: Option<String>,

    #[serde(rename = "package-path", default, skip_serializing_if = "Option::is_none")]
    pub package_path: Option<String>,

    #[serde(rename = "secure-http", default, skip_serializing_if = "Option::is_none")]
    pub secure_http: Option<bool>,

    #[serde(rename = "svn-cache-credentials", default, skip_serializing_if = "Option::is_none")]
    pub svn_cache_credentials: Option<bool>,

    #[serde(rename = "tags-path", default, skip_serializing_if = "Option::is_none")]
    pub tags_path: Option<VcsRepositoryTagsPath>,

    #[serde(rename = "trunk-path", default, skip_serializing_if = "Option::is_none")]
    pub trunk_path: Option<VcsRepositoryTrunkPath>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unique_perforce_client_name: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum VcsRepositoryBranchesPath {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum VcsRepositoryTagsPath {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum VcsRepositoryTrunkPath {
    Boolean(bool),
    String(String),
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VcsRepositoryType {
    #[serde(rename = "vcs")]
    Vcs,
    #[serde(rename = "github")]
    Github,
    #[serde(rename = "git")]
    Git,
    #[serde(rename = "gitlab")]
    Gitlab,
    #[serde(rename = "bitbucket")]
    Bitbucket,
    #[serde(rename = "git-bitbucket")]
    GitBitbucket,
    #[serde(rename = "hg")]
    Hg,
    #[serde(rename = "fossil")]
    Fossil,
    #[serde(rename = "perforce")]
    Perforce,
    #[serde(rename = "svn")]
    Svn,
}

const fn default_composer_package_php_ext_priority() -> usize {
    80
}

const fn default_composer_package_php_ext_support_nts() -> bool {
    true
}

const fn default_composer_package_php_ext_support_zts() -> bool {
    true
}
