<?php

const OAUTH_SIG_METHOD_RSASHA1 = 'RSA-SHA1';

const OAUTH_SIG_METHOD_HMACSHA1 = 'HMAC-SHA1';

const OAUTH_SIG_METHOD_HMACSHA256 = 'HMAC-SHA256';

const OAUTH_AUTH_TYPE_AUTHORIZATION = 3;

const OAUTH_AUTH_TYPE_NONE = 2;

const OAUTH_AUTH_TYPE_URI = 1;

const OAUTH_AUTH_TYPE_FORM = 2;

const OAUTH_HTTP_METHOD_GET = 'GET';

const OAUTH_HTTP_METHOD_POST = 'POST';

const OAUTH_HTTP_METHOD_PUT = 'PUT';

const OAUTH_HTTP_METHOD_HEAD = 'HEAD';

const OAUTH_HTTP_METHOD_DELETE = 'DELETE';

const OAUTH_REQENGINE_STREAMS = 1;

const OAUTH_REQENGINE_CURL = 2;

const OAUTH_OK = 0;

const OAUTH_BAD_NONCE = 4;

const OAUTH_BAD_TIMESTAMP = 8;

const OAUTH_CONSUMER_KEY_UNKNOWN = 16;

const OAUTH_CONSUMER_KEY_REFUSED = 32;

const OAUTH_INVALID_SIGNATURE = 64;

const OAUTH_TOKEN_USED = 128;

const OAUTH_TOKEN_EXPIRED = 256;

const OAUTH_TOKEN_REJECTED = 1024;

const OAUTH_VERIFIER_INVALID = 2048;

const OAUTH_PARAMETER_ABSENT = 4096;

const OAUTH_SIGNATURE_METHOD_REJECTED = 8192;

/**
 * @param string $http_method
 * @param string $uri
 * @param array $request_parameters
 *
 * @return string
 */
function oauth_get_sbs($http_method, $uri, $request_parameters = [])
{
}

/**
 * @param string $uri
 * @return string
 */
function oauth_urlencode($uri)
{
}

class OAuth
{
    /**
     * @var bool
     */
    public $debug;

    /**
     * @var bool
     */
    public $sslChecks;

    /**
     * @var array
     */
    public $debugInfo;

    /**
     * @param string $consumer_key
     * @param string $consumer_secret
     * @param string $signature_method
     * @param int $auth_type
     *
     * @throws OAuthException
     */
    public function __construct(
        $consumer_key,
        $consumer_secret,
        $signature_method = OAUTH_SIG_METHOD_HMACSHA1,
        $auth_type = OAUTH_AUTH_TYPE_AUTHORIZATION,
    ) {}

    /**
     * @return bool
     */
    public function disableDebug()
    {
    }

    /**
     * @return void
     */
    public function disableRedirects()
    {
    }

    /**
     * @return bool
     */
    public function disableSSLChecks()
    {
    }

    /**
     * @return bool
     */
    public function enableDebug()
    {
    }

    /**
     * @return bool
     */
    public function enableRedirects()
    {
    }

    /**
     * @return bool
     */
    public function enableSSLChecks()
    {
    }

    /**
     * @param int $timeout
     *
     * @return void
     */
    public function setTimeout($timeout)
    {
    }

    /**
     * @param string $protected_resource_url
     * @param array $extra_parameters
     * @param string $http_method
     * @param array $http_headers
     *
     * @return mixed
     *
     * @throws OAuthException
     */
    public function fetch($protected_resource_url, $extra_parameters = [], $http_method = null, $http_headers = [])
    {
    }

    /**
     * @param string $access_token_url
     * @param string $auth_session_handle
     * @param string $verifier_token
     *
     * @return array
     *
     * @throws OAuthException
     */
    public function getAccessToken($access_token_url, $auth_session_handle = null, $verifier_token = null)
    {
    }

    /**
     * @return array
     */
    public function getCAPath()
    {
    }

    /**
     * @return string
     */
    public function getLastResponse()
    {
    }

    /**
     * @return string|false
     */
    public function getLastResponseHeaders()
    {
    }

    /**
     * @return array
     */
    public function getLastResponseInfo()
    {
    }

    /**
     * @param string $http_method
     * @param string $url
     * @param mixed  $extra_parameters
     *
     * @return string|false
     */
    public function getRequestHeader($http_method, $url, $extra_parameters = '')
    {
    }

    /**
     * @param string $request_token_url
     * @param string $callback_url
     * @param string $http_method
     *
     * @return array
     *
     * @throws OAuthException
     */
    public function getRequestToken($request_token_url, $callback_url = null, $http_method = 'GET')
    {
    }

    /**
     * @param int $auth_type
     *
     * @return bool
     */
    public function setAuthType($auth_type)
    {
    }

    /**
     * @param string $ca_path
     * @param string $ca_info
     *
     * @return mixed
     */
    public function setCAPath($ca_path = null, $ca_info = null)
    {
    }

    /**
     * @param string $nonce
     *
     * @return mixed
     */
    public function setNonce($nonce)
    {
    }

    /**
     * @param int $reqengine
     *
     * @return void
     */
    public function setRequestEngine($reqengine)
    {
    }

    /**
     * @param string $cert
     *
     * @return mixed
     */
    public function setRSACertificate($cert)
    {
    }

    /**
     * @param string $timestamp
     *
     * @return mixed
     */
    public function setTimestamp($timestamp)
    {
    }

    /**
     * @param string $token
     * @param string $token_secret
     *
     * @return bool
     */
    public function setToken($token, $token_secret)
    {
    }

    /**
     * @param string $version
     *
     * @return bool
     */
    public function setVersion($version)
    {
    }
}

class OAuthException extends Exception
{
    /**
     * @var string
     */
    public $lastResponse;

    /**
     * @var array
     */
    public $debugInfo;
}

class OAuthProvider
{
    /**
     * @param string $req_params
     *
     * @return bool
     */
    final public function addRequiredParameter($req_params)
    {
    }

    /**
     * @return void
     */
    public function callconsumerHandler()
    {
    }

    /**
     * @return void
     */
    public function callTimestampNonceHandler()
    {
    }

    /**
     * @return void
     */
    public function calltokenHandler()
    {
    }

    /**
     * @param string $uri
     * @param string $method
     *
     * @return void
     */
    public function checkOAuthRequest($uri = '', $method = '')
    {
    }

    /**
     * @param array $params_array
     */
    public function __construct($params_array) {}

    /**
     * @param callable $callback_function
     *
     * @return void
     */
    public function consumerHandler($callback_function)
    {
    }

    /**
     * @param int $size
     * @param bool $strong
     *
     * @return string
     */
    final public static function generateToken($size, $strong = false)
    {
    }

    /**
     * @param mixed $params_array
     *
     * @return void
     */
    public function is2LeggedEndpoint($params_array)
    {
    }

    /**
     * @param bool $will_issue_request_token
     *
     * @return void
     */
    public function isRequestTokenEndpoint($will_issue_request_token)
    {
    }

    /**
     * @param string $req_params
     *
     * @return bool
     */
    final public function removeRequiredParameter($req_params)
    {
    }

    /**
     * @param string $oauthexception
     * @param bool $send_headers
     *
     * @return string
     */
    final public static function reportProblem($oauthexception, $send_headers = true)
    {
    }

    /**
     * @param string $param_key
     * @param mixed $param_val
     *
     * @return bool
     */
    final public function setParam($param_key, $param_val = null)
    {
    }

    /**
     * @param string $path
     *
     * @return bool
     */
    final public function setRequestTokenPath($path)
    {
    }

    /**
     * @param callable $callback_function
     *
     * @return void
     */
    public function timestampNonceHandler($callback_function)
    {
    }

    /**
     * @param callable $callback_function
     *
     * @return void
     */
    public function tokenHandler($callback_function)
    {
    }
}
