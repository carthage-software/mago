<?php

namespace http;

use Countable;
use SplObjectStorage;
use SplSubject;

class Client implements SplSubject, Countable
{
    public const DEBUG_INFO = 0;

    public const DEBUG_IN = 1;

    public const DEBUG_OUT = 2;

    public const DEBUG_HEADER = 16;

    public const DEBUG_BODY = 32;

    public const DEBUG_SSL = 64;

    /**
     * @var SplObjectStorage
     */
    private $observers = null;

    /**
     * @var array
     */
    protected $options = null;

    /**
     * @var Message
     */
    protected $history = null;

    /**
     * @var bool
     */
    public $recordHistory = false;

    /**
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @throws Exception\RuntimeException
     */
    public function __construct(string $driver = null, string $persistent_handle_id = null) {}

    /**
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function addCookies(array $cookies = null)
    {
    }

    /**
     * Add specific SSL options.
     * See Client::setSslOptions(), Client::setOptions() and Client\Curl\$ssl options.
     *
     * @param array $ssl_options Add this SSL options.
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function addSslOptions(array $ssl_options = null)
    {
    }

    /**
     * Implements SplSubject. Attach another observer.
     * Attached observers will be notified with progress of each transfer.
     *
     * @param \SplObserver $observer An implementation of SplObserver.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     */
    public function attach(\SplObserver $observer)
    {
    }

    /**
     * Configure the client's low level options.
     *
     * ***NOTE:***
     * This method has been added in v2.3.0.
     *
     * @param array $configuration Key/value pairs of low level options.
     *    See f.e. the [configuration options for the Curl driver](http/Client/Curl#Configuration:).
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     */
    public function configure(array $configuration)
    {
    }

    /**
     * Implements Countable. Retrieve the number of enqueued requests.
     *
     * ***NOTE:***
     * The enqueued requests are counted without regard whether they are finished or not.
     *
     * @return int number of enqueued requests.
     */
    public function count()
    {
    }

    /**
     * Dequeue the Client\Request $request.
     *
     * See Client::requeue(), if you want to requeue the request, instead of calling Client::dequeue() and then Client::enqueue().
     *
     * @param Client\Request $request The request to cancel.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @throws Exception\RuntimeException
     * @return Client
     */
    public function dequeue(Client\Request $request)
    {
    }

    /**
     * Implements SplSubject. Detach $observer, which has been previously attached.
     *
     * @param \SplObserver $observer Previously attached instance of SplObserver implementation.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     */
    public function detach(\SplObserver $observer)
    {
    }

    /**
     * Enable usage of an event library like libevent, which might improve performance with big socket sets.
     *
     * @param bool $enable Whether to enable libevent usage.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     * @see Client::configure()
     */
    #[Deprecated('This method has been deprecated in 2.3.0. Use Client::configure() instead')]
    public function enableEvents(bool $enable = true)
    {
    }

    /**
     * Enable sending pipelined requests to the same host if the driver supports it.
     *
     * @param bool $enable Whether to enable pipelining.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     * @see Client::configure()
     */
    #[Deprecated('This method has been deprecated in 2.3.0. Use Client::configure() instead')]
    public function enablePipelining(bool $enable = true)
    {
    }

    /**
     * Add another Client\Request to the request queue.
     * If the optional callback $cb returns true, the request will be automatically dequeued.
     *
     * ***Note:***
     * The Client\Response object resulting from the request is always stored
     * internally to be retrieved at a later time, __even__ when $cb is used.
     *
     * If you are about to send a lot of requests and do __not__ need the response
     * after executing the callback, you can use Client::getResponse() within
     * the callback to keep the memory usage level as low as possible.
     *
     * See Client::dequeue() and Client::send().
     *
     * @param Client\Request $request The request to enqueue.
     * @param callable $cb as function(Response $response) : ?bool
     *   A callback to automatically call when the request has finished.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @throws Exception\RuntimeException
     * @return Client
     */
    public function enqueue(Client\Request $request, callable $cb = null)
    {
    }

    /**
     * Get a list of available configuration options and their default values.
     *
     * See f.e. the [configuration options for the Curl driver](http/Client/Curl#Configuration:).
     *
     * @throws Exception\InvalidArgumentException
     * @return array list of key/value pairs of available configuration options and their default values.
     */
    public function getAvailableConfiguration()
    {
    }

    /**
     * List available drivers.
     *
     * @return array list of supported drivers.
     */
    public function getAvailableDrivers()
    {
    }

    /**
     * Retrieve a list of available request options and their default values.
     *
     * See f.e. the [request options for the Curl driver](http/Client/Curl#Options:).
     *
     * @throws Exception\InvalidArgumentException
     * @return array list of key/value pairs of available request options and their default values.
     */
    public function getAvailableOptions()
    {
    }

    /**
     * Get priorly set custom cookies.
     * See Client::setCookies().
     *
     * @return array custom cookies.
     */
    public function getCookies()
    {
    }

    /**
     * Simply returns the Message chain representing the request/response history.
     *
     * ***NOTE:***
     * The history is only recorded while Client::$recordHistory is true.
     *
     * @throws Exception\InvalidArgumentException
     * @return Message the request/response message chain representing the client's history.
     */
    public function getHistory()
    {
    }

    /**
     * Returns the SplObjectStorage holding attached observers.
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return \SplObjectStorage observer storage.
     */
    public function getObservers()
    {
    }

    /**
     * Get priorly set options.
     * See Client::setOptions().
     *
     * @return array options.
     */
    public function getOptions()
    {
    }

    /**
     * Retrieve the progress information for $request.
     *
     * @param Client\Request $request The request to retrieve the current progress information for.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return object|null object stdClass instance holding progress information.
     * 		 or NULL if $request is not enqueued.
     */
    public function getProgressInfo(Client\Request $request)
    {
    }

    /**
     * Retrieve the corresponding response of an already finished request, or the last received response if $request is not set.
     *
     * ***NOTE:***
     * If $request is NULL, then the response is removed from the internal storage (stack-like operation).
     *
     * @param Client\Request $request The request to fetch the stored response for.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client\Response|null Client\Response the stored response for the request, or the last that was received.
     * 		 or NULL if no more response was available to pop, when no $request was given.
     */
    public function getResponse(Client\Request $request = null)
    {
    }

    /**
     * Retrieve priorly set SSL options.
     * See Client::getOptions() and Client::setSslOptions().
     *
     * @return array SSL options.
     */
    public function getSslOptions()
    {
    }

    /**
     * Get transfer related information for a running or finished request.
     *
     * @param Client\Request $request The request to probe for transfer info.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return object stdClass instance holding transfer related information.
     */
    public function getTransferInfo(Client\Request $request)
    {
    }

    /**
     * Implements SplSubject. Notify attached observers about progress with $request.
     *
     * @param Client\Request $request The request to notify about.
     * @param object $progress stdClass instance holding progress information.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client
     */
    public function notify(Client\Request $request = null, $progress = null)
    {
    }

    /**
     * Perform outstanding transfer actions.
     * See Client::wait() for the completing interface.
     *
     * @return bool true if there are more transfers to complete.
     */
    public function once()
    {
    }

    /**
     * Requeue an Client\Request.
     *
     * The difference simply is, that this method, in contrast to Client::enqueue(), does not throw an Exception when the request to queue is already enqueued and dequeues it automatically prior enqueueing it again.
     *
     * @param Client\Request $request The request to queue.
     * @param callable $cb as function(Response $response) : ?bool
     *   A callback to automatically call when the request has finished.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     * @return Client
     */
    public function requeue(Client\Request $request, callable $cb = null)
    {
    }

    /**
     * Reset the client to the initial state.
     *
     * @return Client
     */
    public function reset()
    {
    }

    /**
     * Send all enqueued requests.
     * See Client::once() and Client::wait() for a more fine grained interface.
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     * @return Client
     */
    public function send()
    {
    }

    /**
     * Set custom cookies.
     * See Client::addCookies() and Client::getCookies().
     *
     * @param array $cookies Set the custom cookies to this array.
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function setCookies(array $cookies = null)
    {
    }

    /**
     * Set client debugging callback.
     *
     * ***NOTE:***
     * This method has been added in v2.6.0, resp. v3.1.0.
     *
     * @param callable $callback as function(http\Client $c, Client\Request $r, int $type, string $data)
     *   The debug callback. For $type see Client::DEBUG_* constants.
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function setDebug(callable $callback)
    {
    }

    /**
     * Set client options.
     * See Client\Curl.
     *
     * ***NOTE:***
     * Only options specified prior enqueueing a request are applied to the request.
     *
     * @param array $options The options to set.
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function setOptions(array $options = null)
    {
    }

    /**
     * Specifically set SSL options.
     * See Client::setOptions() and Client\Curl\$ssl options.
     *
     * @param array $ssl_options Set SSL options to this array.
     * @throws Exception\InvalidArgumentException
     * @return Client
     */
    public function setSslOptions(array $ssl_options = null)
    {
    }

    /**
     * Wait for $timeout seconds for transfers to provide data.
     * This is the completion call to Client::once().
     *
     * @param float $timeout Seconds to wait for data on open sockets.
     * @return bool
     */
    public function wait(float $timeout = 0)
    {
    }
}

/**
 * A class representing a list of cookies with specific attributes.
 */
class Cookie
{
    /**
     * Do not decode cookie contents.
     */
    public const PARSE_RAW = 1;

    /**
     * The cookies' flags have the secure attribute set.
     */
    public const SECURE = 16;

    /**
     * The cookies' flags have the httpOnly attribute set.
     */
    public const HTTPONLY = 32;

    /**
     * Create a new cookie list.
     *
     * @param mixed $cookies The string or list of cookies to parse or set.
     * @param int $flags Parse flags. See Cookie::PARSE_* constants.
     * @param array $allowed_extras List of extra attribute names to recognize.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     */
    public function __construct($cookies = null, int $flags = 0, array $allowed_extras = null) {}

    /**
     * String cast handler. Alias of Cookie::toString().
     *
     * @return string the cookie(s) represented as string.
     */
    public function __toString()
    {
    }

    /**
     * Add a cookie.
     * See Cookie::setCookie() and Cookie::addCookies().
     *
     * @param string $cookie_name The key of the cookie.
     * @param string $cookie_value The value of the cookie.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function addCookie(string $cookie_name, string $cookie_value)
    {
    }

    /**
     * (Re)set the cookies.
     * See Cookie::setCookies().
     *
     * @param array $cookies Add cookies of this array of form ["name" => "value"].
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function addCookies(array $cookies)
    {
    }

    /**
     * Add an extra attribute to the cookie list.
     * See Cookie::setExtra().
     *
     * @param string $extra_name The key of the extra attribute.
     * @param string $extra_value The value of the extra attribute.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function addExtra(string $extra_name, string $extra_value)
    {
    }

    /**
     * Add several extra attributes.
     * See Cookie::addExtra().
     *
     * @param array $extras A list of extra attributes of the form ["key" => "value"].
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function addExtras(array $extras)
    {
    }

    /**
     * Retrieve a specific cookie value.
     * See Cookie::setCookie().
     *
     * @param string $cookie_name The key of the cookie to look up.
     * @return string|null string the cookie value.
     * 		 or NULL if $cookie_name could not be found.
     */
    public function getCookie(string $cookie_name)
    {
    }

    /**
     * Get the list of cookies.
     * See Cookie::setCookies().
     *
     * @return array the list of cookies of form ["name" => "value"].
     */
    public function getCookies()
    {
    }

    /**
     * Retrieve the effective domain of the cookie list.
     * See Cookie::setDomain().
     *
     * @return string the effective domain.
     */
    public function getDomain()
    {
    }

    /**
     * Get the currently set expires attribute.
     * See Cookie::setExpires().
     *
     * ***NOTE:***
     * A return value of -1 means that the attribute is not set.
     *
     * @return int the currently set expires attribute as seconds since the epoch.
     */
    public function getExpires()
    {
    }

    /**
     * Retrieve an extra attribute.
     * See Cookie::setExtra().
     *
     * @param string $name The key of the extra attribute.
     * @return string the value of the extra attribute.
     */
    public function getExtra(string $name)
    {
    }

    /**
     * Retrieve the list of extra attributes.
     * See Cookie::setExtras().
     *
     * @return array the list of extra attributes of the form ["key" => "value"].
     */
    public function getExtras()
    {
    }

    /**
     * Get the currently set flags.
     * See Cookie::SECURE and Cookie::HTTPONLY constants.
     *
     * @return int the currently set flags bitmask.
     */
    public function getFlags()
    {
    }

    /**
     * Get the currently set max-age attribute of the cookie list.
     * See Cookie::setMaxAge().
     *
     * ***NOTE:***
     * A return value of -1 means that the attribute is not set.
     *
     * @return int the currently set max-age.
     */
    public function getMaxAge()
    {
    }

    /**
     * Retrieve the path the cookie(s) of this cookie list are effective at.
     * See Cookie::setPath().
     *
     * @return string the effective path.
     */
    public function getPath()
    {
    }

    /**
     * (Re)set a cookie.
     * See Cookie::addCookie() and Cookie::setCookies().
     *
     * ***NOTE:***
     * The cookie will be deleted from the list if $cookie_value is NULL.
     *
     * @param string $cookie_name The key of the cookie.
     * @param string $cookie_value The value of the cookie.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setCookie(string $cookie_name, string $cookie_value)
    {
    }

    /**
     * (Re)set the cookies.
     * See Cookie::addCookies().
     *
     * @param array $cookies Set the cookies to this array.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setCookies(array $cookies = null)
    {
    }

    /**
     * Set the effective domain of the cookie list.
     * See Cookie::setPath().
     *
     * @param string $value The domain the cookie(s) belong to.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setDomain(string $value = null)
    {
    }

    /**
     * Set the traditional expires timestamp.
     * See Cookie::setMaxAge() for a safer alternative.
     *
     * @param int $value The expires timestamp as seconds since the epoch.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setExpires(int $value = -1)
    {
    }

    /**
     * (Re)set an extra attribute.
     * See Cookie::addExtra().
     *
     * ***NOTE:***
     * The attribute will be removed from the extras list if $extra_value is NULL.
     *
     * @param string $extra_name The key of the extra attribute.
     * @param string $extra_value The value of the extra attribute.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setExtra(string $extra_name, string $extra_value = null)
    {
    }

    /**
     * (Re)set the extra attributes.
     * See Cookie::addExtras().
     *
     * @param array $extras Set the extra attributes to this array.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setExtras(array $extras = null)
    {
    }

    /**
     * Set the flags to specified $value.
     * See Cookie::SECURE and Cookie::HTTPONLY constants.
     *
     * @param int $value The new flags bitmask.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setFlags(int $value = 0)
    {
    }

    /**
     * Set the maximum age the cookie may have on the client side.
     * This is a client clock departure safe alternative to the "expires" attribute.
     * See Cookie::setExpires().
     *
     * @param int $value The max-age in seconds.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setMaxAge(int $value = -1)
    {
    }

    /**
     * Set the path the cookie(s) of this cookie list should be effective at.
     * See Cookie::setDomain().
     *
     * @param string $path The URL path the cookie(s) should take effect within.
     * @throws Exception\InvalidArgumentException
     * @return Cookie self.
     */
    public function setPath(string $path = null)
    {
    }

    /**
     * Get the cookie list as array.
     *
     * @return array the cookie list as array.
     */
    public function toArray()
    {
    }

    /**
     * Retrieve the string representation of the cookie list.
     * See Cookie::toArray().
     *
     * @return string the cookie list as string.
     */
    public function toString()
    {
    }
}

namespace Encoding;



namespace http;

/**
 * The Env class provides static methods to manipulate and inspect the server's current request's HTTP environment.
 */
class Env
{
    /**
     * Retrieve the current HTTP request's body.
     *
     * @param string $body_class_name A user class extending Message\Body.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Message\Body instance representing the request body
     */
    public function getRequestBody(string $body_class_name = null)
    {
    }

    /**
     * Retrieve one or all headers of the current HTTP request.
     *
     * @param string $header_name The key of a header to retrieve.
     * @return string|null|array NULL if $header_name was not found
     * 		 or string the compound header when $header_name was found
     * 		 or array of all headers if $header_name was not specified
     */
    public function getRequestHeader(string $header_name = null)
    {
    }

    /**
     * Get the HTTP response code to send.
     *
     * @return int the HTTP response code.
     */
    public function getResponseCode()
    {
    }

    /**
     * Get one or all HTTP response headers to be sent.
     *
     * @param string $header_name The name of the response header to retrieve.
     * @return string|array|null string the compound value of the response header to send
     * 		 or NULL if the header was not found
     * 		 or array of all response headers, if $header_name was not specified
     */
    public function getResponseHeader(string $header_name = null)
    {
    }

    /**
     * Retrieve a list of all known HTTP response status.
     *
     * @return array mapping of the form \[
     *   ...
     *   int $code => string $status
     *   ...
     *   \]
     */
    public function getResponseStatusForAllCodes()
    {
    }

    /**
     * Retrieve the string representation of specified HTTP response code.
     *
     * @param int $code The HTTP response code to get the string representation for.
     * @return string the HTTP response status message (may be empty, if no message for this code was found)
     */
    public function getResponseStatusForCode(int $code)
    {
    }

    /**
     * Generic negotiator. For specific client negotiation see Env::negotiateContentType() and related methods.
     *
     * ***NOTE:***
     * The first element of $supported serves as a default if no operand matches.
     *
     * @param string $params HTTP header parameter's value to negotiate.
     * @param array $supported List of supported negotiation operands.
     * @param string $prim_typ_sep A "primary type separator", i.e. that would be a hyphen for content language negotiation (en-US, de-DE, etc.).
     * @param array &$result Out parameter recording negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the closest match negotiated, or the default (first entry of $supported).
     */
    public function negotiate(string $params, array $supported, string $prim_typ_sep = null, array &$result = null)
    {
    }

    /**
     * Negotiate the client's preferred character set.
     *
     * ***NOTE:***
     * The first element of $supported character sets serves as a default if no character set matches.
     *
     * @param array $supported List of supported content character sets.
     * @param array &$result Out parameter recording negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the negotiated character set.
     */
    public function negotiateCharset(array $supported, array &$result = null)
    {
    }

    /**
     * Negotiate the client's preferred MIME content type.
     *
     * ***NOTE:***
     * The first element of $supported content types serves as a default if no content-type matches.
     *
     * @param array $supported List of supported MIME content types.
     * @param array &$result Out parameter recording negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the negotiated content type.
     */
    public function negotiateContentType(array $supported, array &$result = null)
    {
    }

    /**
     * Negotiate the client's preferred encoding.
     *
     * ***NOTE:***
     * The first element of $supported encodings serves as a default if no encoding matches.
     *
     * @param array $supported List of supported content encodings.
     * @param array &$result Out parameter recording negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the negotiated encoding.
     */
    public function negotiateEncoding(array $supported, array &$result = null)
    {
    }

    /**
     * Negotiate the client's preferred language.
     *
     * ***NOTE:***
     * The first element of $supported languages serves as a default if no language matches.
     *
     * @param array $supported List of supported content languages.
     * @param array &$result Out parameter recording negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the negotiated language.
     */
    public function negotiateLanguage(array $supported, array &$result = null)
    {
    }

    /**
     * Set the HTTP response code to send.
     *
     * @param int $code The HTTP response status code.
     * @return bool Success.
     */
    public function setResponseCode(int $code)
    {
    }

    /**
     * Set a response header, either replacing a prior set header, or appending the new header value, depending on $replace.
     *
     * If no $header_value is specified, or $header_value is NULL, then a previously set header with the same key will be deleted from the list.
     *
     * If $response_code is not 0, the response status code is updated accordingly.
     *
     * @param string $header_name
     * @param mixed $header_value
     * @param int $response_code
     * @param bool $replace
     * @return bool Success.
     */
    public function setResponseHeader(
        string $header_name,
        $header_value = null,
        int $response_code = null,
        bool $replace = null,
    ) {
    }
}

/**
 * The http extension's Exception interface.
 *
 * Use it to catch any Exception thrown by pecl/http.
 *
 * The individual exception classes extend their equally named native PHP extensions, if such exist, and implement this empty interface. For example the Exception\BadMethodCallException extends SPL's BadMethodCallException.
 */
interface Exception
{
}

/**
 * The Header class provides methods to manipulate, match, negotiate and serialize HTTP headers.
 */
class Header implements \Serializable
{
    /**
     * None of the following match constraints applies.
     */
    public const MATCH_LOOSE = 0;

    /**
     * Perform case sensitive matching.
     */
    public const MATCH_CASE = 1;

    /**
     * Match only on word boundaries (according by CType alpha-numeric).
     */
    public const MATCH_WORD = 16;

    /**
     * Match the complete string.
     */
    public const MATCH_FULL = 32;

    /**
     * Case sensitively match the full string (same as MATCH_CASE|MATCH_FULL).
     */
    public const MATCH_STRICT = 33;

    /**
     * The name of the HTTP header.
     *
     * @var string
     */
    public $name = null;

    /**
     * The value of the HTTP header.
     *
     * @var mixed
     */
    public $value = null;

    /**
     * Create an Header instance for use of simple matching or negotiation. If the value of the header is an array it may be compounded to a single comma separated string.
     *
     * @param string $name The HTTP header name.
     * @param mixed $value The value of the header.
     *
     * # Throws:
     */
    public function __construct(string $name = null, $value = null) {}

    /**
     * String cast handler. Alias of Header::serialize().
     *
     * @return string the serialized form of the HTTP header (i.e. "Name: value").
     */
    public function __toString()
    {
    }

    /**
     * Create a parameter list out of the HTTP header value.
     *
     * @param mixed $ps The parameter separator(s).
     * @param mixed $as The argument separator(s).
     * @param mixed $vs The value separator(s).
     * @param int $flags The modus operandi. See Params constants.
     * @return Params instance
     */
    public function getParams($ps = null, $as = null, $vs = null, int $flags = null)
    {
    }

    /**
     * Match the HTTP header's value against provided $value according to $flags.
     *
     * @param string $value The comparison value.
     * @param int $flags The modus operandi. See Header constants.
     * @return bool whether $value matches the header value according to $flags.
     */
    public function match(string $value, int $flags = null)
    {
    }

    /**
     * Negotiate the header's value against a list of supported values in $supported.
     * Negotiation operation is adopted according to the header name, i.e. if the
     * header being negotiated is Accept, then a slash is used as primary type
     * separator, and if the header is Accept-Language respectively, a hyphen is
     * used instead.
     *
     * ***NOTE:***
     * The first element of $supported serves as a default if no operand matches.
     *
     * @param array $supported The list of supported values to negotiate.
     * @param array &$result Out parameter recording the negotiation results.
     * @return string|null NULL if negotiation fails.
     * 		 or string the closest match negotiated, or the default (first entry of $supported).
     */
    public function negotiate(array $supported, array &$result = null)
    {
    }

    /**
     * Parse HTTP headers.
     * See also Header\Parser.
     *
     * @param string $header The complete string of headers.
     * @param string $header_class A class extending Header.
     * @return array|false array of parsed headers, where the elements are instances of $header_class if specified.
     * 		 or false if parsing fails.
     */
    public function parse(string $header, string $header_class = null)
    {
    }

    /**
     * Implements Serializable.
     *
     * @return string serialized representation of HTTP header (i.e. "Name: value")
     */
    public function serialize()
    {
    }

    /**
     * Convenience method. Alias of Header::serialize().
     *
     * @return string the serialized form of the HTTP header (i.e. "Name: value").
     */
    public function toString()
    {
    }

    /**
     * Implements Serializable.
     *
     * @param string $serialized The serialized HTTP header (i.e. "Name: value")
     */
    public function unserialize($serialized)
    {
    }
}

/**
 * The message class builds the foundation for any request and response message.
 *
 * See Client\Request and Client\Response, as well as Env\Request and Env\Response.
 */
class Message implements \Countable, \Serializable, \Iterator
{
    /**
     * No specific type of message.
     */
    public const TYPE_NONE = 0;

    /**
     * A request message.
     */
    public const TYPE_REQUEST = 1;

    /**
     * A response message.
     */
    public const TYPE_RESPONSE = 2;

    /**
     * The message type. See Message::TYPE_* constants.
     *
     * @var int
     */
    protected $type = Message::TYPE_NONE;

    /**
     * The message's body.
     *
     * @var Message\Body
     */
    protected $body = null;

    /**
     * The request method if the message is of type request.
     *
     * @var string
     */
    protected $requestMethod = '';

    /**
     * The request url if the message is of type request.
     *
     * @var string
     */
    protected $requestUrl = '';

    /**
     * The response status phrase if the message is of type response.
     *
     * @var string
     */
    protected $responseStatus = '';

    /**
     * The response code if the message is of type response.
     *
     * @var int
     */
    protected $responseCode = 0;

    /**
     * A custom HTTP protocol version.
     *
     * @var string
     */
    protected $httpVersion = null;

    /**
     * Any message headers.
     *
     * @var array
     */
    protected $headers = null;

    /**
     * Any parent message.
     *
     * @var Message
     */
    protected $parentMessage;

    /**
     * Create a new HTTP message.
     *
     * @param mixed $message Either a resource or a string, representing the HTTP message.
     * @param bool $greedy Whether to read from a $message resource until EOF.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMessageException
     */
    public function __construct($message = null, bool $greedy = true) {}

    /**
     * Retrieve the message serialized to a string.
     * Alias of Message::toString().
     *
     * @return string the single serialized HTTP message.
     */
    public function __toString()
    {
    }

    /**
     * Append the data of $body to the message's body.
     * See Message::setBody() and Message\Body::append().
     *
     * @param Message\Body $body The message body to add.
     * @return Message self.
     */
    public function addBody(Message\Body $body)
    {
    }

    /**
     * Add an header, appending to already existing headers.
     * See Message::addHeaders() and Message::setHeader().
     *
     * @param string $name The header name.
     * @param mixed $value The header value.
     * @return Message self.
     */
    public function addHeader(string $name, $value)
    {
    }

    /**
     * Add headers, optionally appending values, if header keys already exist.
     * See Message::addHeader() and Message::setHeaders().
     *
     * @param array $headers The HTTP headers to add.
     * @param bool $append Whether to append values for existing headers.
     * @return Message self.
     */
    public function addHeaders(array $headers, bool $append = false)
    {
    }

    /**
     * Implements Countable.
     *
     * @return int the count of messages in the chain above the current message.
     */
    public function count()
    {
    }

    /**
     * Implements iterator.
     * See Message::valid() and Message::rewind().
     *
     * @return Message the current message in the iterated message chain.
     */
    public function current()
    {
    }

    /**
     * Detach a clone of this message from any message chain.
     *
     * @throws Exception\InvalidArgumentException
     * @return Message clone.
     */
    public function detach()
    {
    }

    /**
     * Retrieve the message's body.
     * See Message::setBody().
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Message\Body the message body.
     */
    public function getBody()
    {
    }

    /**
     * Retrieve a single header, optionally hydrated into a Header extending class.
     *
     * @param string $header The header's name.
     * @param string $into_class The name of a class extending Header.
     * @return mixed|Header mixed the header value if $into_class is NULL.
     * 		 or Header descendant.
     */
    public function getHeader(string $header, string $into_class = null)
    {
    }

    /**
     * Retrieve all message headers.
     * See Message::setHeaders() and Message::getHeader().
     *
     * @return array the message's headers.
     */
    public function getHeaders()
    {
    }

    /**
     * Retrieve the HTTP protocol version of the message.
     * See Message::setHttpVersion().
     *
     * @return string the HTTP protocol version, e.g. "1.0"; defaults to "1.1".
     */
    public function getHttpVersion()
    {
    }

    /**
     * Retrieve the first line of a request or response message.
     * See Message::setInfo and also:
     *
     * * Message::getType()
     * * Message::getHttpVersion()
     * * Message::getResponseCode()
     * * Message::getResponseStatus()
     * * Message::getRequestMethod()
     * * Message::getRequestUrl()
     *
     * @return string|null string the HTTP message information.
     * 		 or NULL if the message is neither of type request nor response.
     */
    public function getInfo()
    {
    }

    /**
     * Retrieve any parent message.
     * See Message::reverse().
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @return Message the parent message.
     */
    public function getParentMessage()
    {
    }

    /**
     * Retrieve the request method of the message.
     * See Message::setRequestMethod() and Message::getRequestUrl().
     *
     * @return string|false string the request method.
     * 		 or false if the message was not of type request.
     */
    public function getRequestMethod()
    {
    }

    /**
     * Retrieve the request URL of the message.
     * See Message::setRequestUrl().
     *
     * @return string|false string the request URL; usually the path and the querystring.
     * 		 or false if the message was not of type request.
     */
    public function getRequestUrl()
    {
    }

    /**
     * Retrieve the response code of the message.
     * See Message::setResponseCode() and Message::getResponseStatus().
     *
     * @return int|false int the response status code.
     * 		 or false if the message is not of type response.
     */
    public function getResponseCode()
    {
    }

    /**
     * Retrieve the response status of the message.
     * See Message::setResponseStatus() and Message::getResponseCode().
     *
     * @return string|false string the response status phrase.
     * 		 or false if the message is not of type response.
     */
    public function getResponseStatus()
    {
    }

    /**
     * Retrieve the type of the message.
     * See Message::setType() and Message::getInfo().
     *
     * @return int the message type. See Message::TYPE_* constants.
     */
    public function getType()
    {
    }

    /**
     * Check whether this message is a multipart message based on it's content type.
     * If the message is a multipart message and a reference $boundary is given, the boundary string of the multipart message will be stored in $boundary.
     *
     * See Message::splitMultipartBody().
     *
     * @param string &$boundary A reference where the boundary string will be stored.
     * @return bool whether this is a message with a multipart "Content-Type".
     */
    public function isMultipart(string &$boundary = null)
    {
    }

    /**
     * Implements Iterator.
     * See Message::current() and Message::rewind().
     *
     * @return int a non-sequential integer key.
     */
    public function key()
    {
    }

    /**
     * Implements Iterator.
     * See Message::valid() and Message::rewind().
     */
    public function next()
    {
    }

    /**
     * Prepend message(s) $message to this message, or the top most message of this message chain.
     *
     * ***NOTE:***
     * The message chains must not overlap.
     *
     * @param Message $message The message (chain) to prepend as parent messages.
     * @param bool $top Whether to prepend to the top-most parent message.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Message self.
     */
    public function prepend(Message $message, bool $top = true)
    {
    }

    /**
     * Reverse the message chain and return the former top-most message.
     *
     * ***NOTE:***
     * Message chains are ordered in reverse-parsed order by default, i.e. the last parsed message is the message you'll receive from any call parsing HTTP messages.
     *
     * This call re-orders the messages of the chain and returns the message that was parsed first with any later parsed messages re-parentized.
     *
     * @throws Exception\InvalidArgumentException
     * @return Message the other end of the message chain.
     */
    public function reverse()
    {
    }

    /**
     * Implements Iterator.
     */
    public function rewind()
    {
    }

    /**
     * Implements Serializable.
     *
     * @return string the serialized HTTP message.
     */
    public function serialize()
    {
    }

    /**
     * Set the message's body.
     * See Message::getBody() and Message::addBody().
     *
     * @param Message\Body $body The new message body.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Message self.
     */
    public function setBody(Message\Body $body)
    {
    }

    /**
     * Set a single header.
     * See Message::getHeader() and Message::addHeader().
     *
     * ***NOTE:***
     * Prior to v2.5.6/v3.1.0 headers with the same name were merged into a single
     * header with values concatenated by comma.
     *
     * @param string $header The header's name.
     * @param mixed $value The header's value. Removes the header if NULL.
     * @return Message self.
     */
    public function setHeader(string $header, $value = null)
    {
    }

    /**
     * Set the message headers.
     * See Message::getHeaders() and Message::addHeaders().
     *
     * ***NOTE:***
     * Prior to v2.5.6/v3.1.0 headers with the same name were merged into a single
     * header with values concatenated by comma.
     *
     * @param array $headers The message's headers.
     * @return Message null.
     */
    public function setHeaders(array $headers = null)
    {
    }

    /**
     * Set the HTTP protocol version of the message.
     * See Message::getHttpVersion().
     *
     * @param string $http_version The protocol version, e.g. "1.1", optionally prefixed by "HTTP/".
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadHeaderException
     * @return Message self.
     */
    public function setHttpVersion(string $http_version)
    {
    }

    /**
     * Set the complete message info, i.e. type and response resp. request information, at once.
     * See Message::getInfo().
     *
     * @param string $http_info The message info (first line of an HTTP message).
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadHeaderException
     * @return Message self.
     */
    public function setInfo(string $http_info)
    {
    }

    /**
     * Set the request method of the message.
     * See Message::getRequestMethod() and Message::setRequestUrl().
     *
     * @param string $method The request method.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @return Message self.
     */
    public function setRequestMethod(string $method)
    {
    }

    /**
     * Set the request URL of the message.
     * See Message::getRequestUrl() and Message::setRequestMethod().
     *
     * @param string $url The request URL.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @return Message self.
     */
    public function setRequestUrl(string $url)
    {
    }

    /**
     * Set the response status code.
     * See Message::getResponseCode() and Message::setResponseStatus().
     *
     * ***NOTE:***
     * This method also resets the response status phrase to the default for that code.
     *
     * @param int $response_code The response code.
     * @param bool $strict Whether to check that the response code is between 100 and 599 inclusive.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @return Message self.
     */
    public function setResponseCode(int $response_code, bool $strict = true)
    {
    }

    /**
     * Set the response status phrase.
     * See Message::getResponseStatus() and Message::setResponseCode().
     *
     * @param string $response_status The status phrase.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @return Message self.
     */
    public function setResponseStatus(string $response_status)
    {
    }

    /**
     * Set the message type and reset the message info.
     * See Message::getType() and Message::setInfo().
     *
     * @param int $type The desired message type. See the Message::TYPE_* constants.
     * @return Message self.
     */
    public function setType(int $type)
    {
    }

    /**
     * Splits the body of a multipart message.
     * See Message::isMultipart() and Message\Body::addPart().
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @throws Exception\BadMessageException
     * @return Message a message chain of all messages of the multipart body.
     */
    public function splitMultipartBody()
    {
    }

    /**
     * Stream the message through a callback.
     *
     * @param callable $callback The callback of the form function(http\Message $from, string $data).
     * @return Message self.
     */
    public function toCallback(callable $callback)
    {
    }

    /**
     * Stream the message into stream $stream, starting from $offset, streaming $maxlen at most.
     *
     * @param resource $stream The resource to write to.
     * @return Message self.
     */
    public function toStream($stream)
    {
    }

    /**
     * Retrieve the message serialized to a string.
     *
     * @param bool $include_parent Whether to include all parent messages.
     * @return string the HTTP message chain serialized to a string.
     */
    public function toString(bool $include_parent = false)
    {
    }

    /**
     * Implements Serializable.
     *
     * @param string $data The serialized message.
     */
    public function unserialize($data)
    {
    }

    /**
     * Implements Iterator.
     * See Message::current() and Message::rewind().
     *
     * @return bool whether Message::current() would not return NULL.
     */
    public function valid()
    {
    }
}

/**
 * Parse, interpret and compose HTTP (header) parameters.
 */
class Params implements \ArrayAccess
{
    /**
     * The default parameter separator (",").
     */
    public const DEF_PARAM_SEP = ',';

    /**
     * The default argument separator (";").
     */
    public const DEF_ARG_SEP = ';';

    /**
     * The default value separator ("=").
     */
    public const DEF_VAL_SEP = '=';

    /**
     * Empty param separator to parse cookies.
     */
    public const COOKIE_PARAM_SEP = '';

    /**
     * Do not interpret the parsed parameters.
     */
    public const PARSE_RAW = 0;

    /**
     * Interpret input as default formatted parameters.
     */
    public const PARSE_DEFAULT = 17;

    /**
     * Parse backslash escaped (quoted) strings.
     */
    public const PARSE_ESCAPED = 1;

    /**
     * Urldecode single units of parameters, arguments and values.
     */
    public const PARSE_URLENCODED = 4;

    /**
     * Parse sub dimensions indicated by square brackets.
     */
    public const PARSE_DIMENSION = 8;

    /**
     * Parse URL querystring (same as Params::PARSE_URLENCODED|http\Params::PARSE_DIMENSION).
     */
    public const PARSE_QUERY = 12;

    /**
     * Parse [RFC5987](http://tools.ietf.org/html/rfc5987) style encoded character set and language information embedded in HTTP header params.
     */
    public const PARSE_RFC5987 = 16;

    /**
     * Parse [RFC5988](http://tools.ietf.org/html/rfc5988) (Web Linking) tags of Link headers.
     */
    public const PARSE_RFC5988 = 32;

    /**
     * The (parsed) parameters.
     *
     * @var array
     */
    public $params = null;

    /**
     * The parameter separator(s).
     *
     * @var array
     */
    public $param_sep = Params::DEF_PARAM_SEP;

    /**
     * The argument separator(s).
     *
     * @var array
     */
    public $arg_sep = Params::DEF_ARG_SEP;

    /**
     * The value separator(s).
     *
     * @var array
     */
    public $val_sep = Params::DEF_VAL_SEP;

    /**
     * The modus operandi of the parser. See Params::PARSE_* constants.
     *
     * @var int
     */
    public $flags = Params::PARSE_DEFAULT;

    /**
     * Instantiate a new HTTP (header) parameter set.
     *
     * @param mixed $params Pre-parsed parameters or a string to be parsed.
     * @param mixed $ps The parameter separator(s).
     * @param mixed $as The argument separator(s).
     * @param mixed $vs The value separator(s).
     * @param int $flags The modus operandi. See Params::PARSE_* constants.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     */
    public function __construct($params = null, $ps = null, $as = null, $vs = null, int $flags = null) {}

    /**
     * String cast handler. Alias of Params::toString().
     * Returns a stringified version of the parameters.
     *
     * @return string version of the parameters.
     */
    public function __toString()
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to look after.
     * @return bool Existence.
     */
    public function offsetExists($name)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to retrieve.
     * @return mixed contents at offset.
     */
    public function offsetGet($name)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to modify.
     * @param mixed $value The value to set.
     */
    public function offsetSet($name, $value)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to delete.
     */
    public function offsetUnset($name)
    {
    }

    /**
     * Convenience method that simply returns Params::$params.
     *
     * @return array of parameters.
     */
    public function toArray()
    {
    }

    /**
     * Returns a stringified version of the parameters.
     *
     * @return string version of the parameters.
     */
    public function toString()
    {
    }
}

/**
 * The QueryString class provides versatile facilities to retrieve, use and manipulate query strings and form data.
 */
class QueryString implements \Serializable, \ArrayAccess, \IteratorAggregate
{
    /**
     * Cast requested value to bool.
     */
    public const TYPE_BOOL = 16;

    /**
     * Cast requested value to int.
     */
    public const TYPE_INT = 4;

    /**
     * Cast requested value to float.
     */
    public const TYPE_FLOAT = 5;

    /**
     * Cast requested value to string.
     */
    public const TYPE_STRING = 6;

    /**
     * Cast requested value to an array.
     */
    public const TYPE_ARRAY = 7;

    /**
     * Cast requested value to an object.
     */
    public const TYPE_OBJECT = 8;

    /**
     * The global instance. See QueryString::getGlobalInstance().
     *
     * @var QueryString
     */
    private $instance = null;

    /**
     * The data.
     *
     * @var array
     */
    private $queryArray = null;

    /**
     * Create an independent querystring instance.
     *
     * @param mixed $params The query parameters to use or parse.
     * @throws Exception\BadQueryStringException
     */
    public function __construct($params = null) {}

    /**
     * Get the string representation of the querystring (x-www-form-urlencoded).
     *
     * @return string the x-www-form-urlencoded querystring.
     */
    public function __toString()
    {
    }

    /**
     * Retrieve an querystring value.
     *
     * See QueryString::TYPE_* constants.
     *
     * @param string $name The key to retrieve the value for.
     * @param mixed $type The type to cast the value to. See QueryString::TYPE_* constants.
     * @param mixed $defval The default value to return if the key $name does not exist.
     * @param bool $delete Whether to delete the entry from the querystring after retrieval.
     * @return QueryString|string|mixed|mixed|string QueryString if called without arguments.
     * 		 or string the whole querystring if $name is of zero length.
     * 		 or mixed $defval if the key $name does not exist.
     * 		 or mixed the querystring value cast to $type if $type was specified and the key $name exists.
     * 		 or string the querystring value if the key $name exists and $type is not specified or equals QueryString::TYPE_STRING.
     */
    public function get(string $name = null, $type = null, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve an array value with at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return array|mixed array the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getArray(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve a boolean value at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return bool|mixed bool the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getBool(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve a float value at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return float|mixed float the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getFloat(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve the global querystring instance referencing $_GET.
     *
     * @throws Exception\UnexpectedValueException
     * @return QueryString the QueryString::$instance
     */
    public function getGlobalInstance()
    {
    }

    /**
     * Retrieve a int value at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return int|mixed int the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getInt(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Implements IteratorAggregate.
     *
     * @throws Exception\InvalidArgumentException
     * @throws \InvalidArgumentException
     */
    public function getIterator()
    {
    }

    /**
     * Retrieve a object value with at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return object|mixed object the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getObject(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve a string value with at offset $name.
     *
     * @param string $name The key to look up.
     * @param mixed $defval The default value to return if the offset $name does not exist.
     * @param bool $delete Whether to remove the key and value from the querystring after retrieval.
     * @return string|mixed string the (casted) value.
     * 		 or mixed $defval if offset $name does not exist.
     */
    public function getString(string $name, $defval = null, bool $delete = false)
    {
    }

    /**
     * Set additional $params to a clone of this instance.
     * See QueryString::set().
     *
     * ***NOTE:***
     * This method returns a clone (copy) of this instance.
     *
     * @param mixed $params Additional params as object, array or string to parse.
     * @throws Exception\BadQueryStringException
     * @return QueryString clone.
     */
    public function mod($params = null)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to look up.
     * @return bool whether the key $name isset.
     */
    public function offsetExists($name)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param mixed $offset The offset to look up.
     * @return mixed|null mixed the value locate at offset $name.
     * 		 or NULL if key $name could not be found.
     */
    public function offsetGet($offset)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The key to set the value for.
     * @param mixed $data The data to place at offset $name.
     */
    public function offsetSet($name, $data)
    {
    }

    /**
     * Implements ArrayAccess.
     *
     * @param string $name The offset to look up.
     */
    public function offsetUnset($name)
    {
    }

    /**
     * Implements Serializable.
     * See QueryString::toString().
     *
     * @return string the x-www-form-urlencoded querystring.
     */
    public function serialize()
    {
    }

    /**
     * Set additional querystring entries.
     *
     * @param mixed $params Additional params as object, array or string to parse.
     * @return QueryString self.
     */
    public function set($params)
    {
    }

    /**
     * Simply returns QueryString::$queryArray.
     *
     * @return array the $queryArray property.
     */
    public function toArray()
    {
    }

    /**
     * Get the string representation of the querystring (x-www-form-urlencoded).
     *
     * @return string the x-www-form-urlencoded querystring.
     */
    public function toString()
    {
    }

    /**
     * Implements Serializable.
     *
     * @param string $serialized The x-www-form-urlencoded querystring.
     * @throws Exception
     */
    public function unserialize($serialized)
    {
    }

    /**
     * Translate character encodings of the querystring with ext/iconv.
     *
     * ***NOTE:***
     * This method is only available when ext/iconv support was enabled at build time.
     *
     * @param string $from_enc The encoding to convert from.
     * @param string $to_enc The encoding to convert to.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadConversionException
     * @return QueryString self.
     */
    public function xlate(string $from_enc, string $to_enc)
    {
    }
}

/**
 * The Url class provides versatile means to parse, construct and manipulate URLs.
 */
class Url
{
    /**
     * Replace parts of the old URL with parts of the new.
     */
    public const REPLACE = 0;

    /**
     * Whether a relative path should be joined into the old path.
     */
    public const JOIN_PATH = 1;

    /**
     * Whether the querystrings should be joined.
     */
    public const JOIN_QUERY = 2;

    /**
     * Strip the user information from the URL.
     */
    public const STRIP_USER = 4;

    /**
     * Strip the password from the URL.
     */
    public const STRIP_PASS = 8;

    /**
     * Strip user and password information from URL (same as STRIP_USER|STRIP_PASS).
     */
    public const STRIP_AUTH = 12;

    /**
     * Do not include the port.
     */
    public const STRIP_PORT = 32;

    /**
     * Do not include the URL path.
     */
    public const STRIP_PATH = 64;

    /**
     * Do not include the URL querystring.
     */
    public const STRIP_QUERY = 128;

    /**
     * Strip the fragment (hash) from the URL.
     */
    public const STRIP_FRAGMENT = 256;

    /**
     * Strip everything except scheme and host information.
     */
    public const STRIP_ALL = 492;

    /**
     * Import initial URL parts from the SAPI environment.
     */
    public const FROM_ENV = 4096;

    /**
     * Whether to sanitize the URL path (consolidate double slashes, directory jumps etc.)
     */
    public const SANITIZE_PATH = 8192;

    /**
     * Parse UTF-8 encoded multibyte sequences.
     */
    public const PARSE_MBUTF8 = 131072;

    /**
     * Parse locale encoded multibyte sequences (on systems with wide character support).
     */
    public const PARSE_MBLOC = 65536;

    /**
     * Parse and convert multibyte hostnames according to IDNA (with IDNA support).
     */
    public const PARSE_TOIDN = 1048576;

    /**
     * Explicitly request IDNA2003 implementation if available (libidn, idnkit or ICU).
     */
    public const PARSE_TOIDN_2003 = 9437184;

    /**
     * Explicitly request IDNA2008 implementation if available (libidn2, idnkit2 or ICU).
     */
    public const PARSE_TOIDN_2008 = 5242880;

    /**
     * Percent encode multibyte sequences in the userinfo, path, query and fragment parts of the URL.
     */
    public const PARSE_TOPCT = 2097152;

    /**
     * Continue parsing when encountering errors.
     */
    public const IGNORE_ERRORS = 268435456;

    /**
     * Suppress errors/exceptions.
     */
    public const SILENT_ERRORS = 536870912;

    /**
     * Standard flags used by default internally for e.g. Message::setRequestUrl().
     *   Enables joining path and query, sanitizing path, multibyte/unicode, international domain names and transient percent encoding.
     */
    public const STDFLAGS = 3350531;

    /**
     * The URL's scheme.
     *
     * @var string
     */
    public $scheme = null;

    /**
     * Authenticating user.
     *
     * @var string
     */
    public $user = null;

    /**
     * Authentication password.
     *
     * @var string
     */
    public $pass = null;

    /**
     * Hostname/domain.
     *
     * @var string
     */
    public $host = null;

    /**
     * Port.
     *
     * @var string
     */
    public $port = null;

    /**
     * URL path.
     *
     * @var string
     */
    public $path = null;

    /**
     * URL querystring.
     *
     * @var string
     */
    public $query = null;

    /**
     * URL fragment (hash).
     *
     * @var string
     */
    public $fragment = null;

    /**
     * Create an instance of an Url.
     *
     * ***NOTE:***
     * Prior to v3.0.0, the default for the $flags parameter was Url::FROM_ENV.
     *
     * See also Env\Url.
     *
     * @param mixed $old_url Initial URL parts. Either an array, object, Url instance or string to parse.
     * @param mixed $new_url Overriding URL parts. Either an array, object, Url instance or string to parse.
     * @param int $flags The modus operandi of constructing the url. See Url constants.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadUrlException
     */
    public function __construct($old_url = null, $new_url = null, int $flags = 0) {}

    /**
     * String cast handler. Alias of Url::toString().
     *
     * @return string the URL as string.
     */
    public function __toString()
    {
    }

    /**
     * Clone this URL and apply $parts to the cloned URL.
     *
     * ***NOTE:***
     * This method returns a clone (copy) of this instance.
     *
     * @param mixed $parts New URL parts.
     * @param int $flags Modus operandi of URL construction. See Url constants.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadUrlException
     * @return Url clone.
     */
    public function mod($parts, int $flags = Url::JOIN_PATH | Url::JOIN_QUERY | Url::SANITIZE_PATH)
    {
    }

    /**
     * Retrieve the URL parts as array.
     *
     * @return array the URL parts.
     */
    public function toArray()
    {
    }

    /**
     * Get the string prepresentation of the URL.
     *
     * @return string the URL as string.
     */
    public function toString()
    {
    }
}

/**
 * The Client\Curl namespace holds option value constants specific to the curl driver of the Client.
 */

namespace Client\Curl;

/**
 * Bitmask of available libcurl features.
 *   See Client\Curl\Features namespace.
 */
const FEATURES = 4179869;

/**
 * List of library versions of or linked into libcurl,
 *   e.g. "libcurl/7.50.0 OpenSSL/1.0.2h zlib/1.2.8 libidn/1.32 nghttp2/1.12.0".
 *   See Client\Curl\Versions namespace.
 */
const VERSIONS = 'libcurl/7.64.0 OpenSSL/1.1.1b zlib/1.2.11 libidn2/2.0.5 libpsl/0.20.2 (+libidn2/2.0.5) libssh2/1.8.0 nghttp2/1.36.0 librtmp/2.3';

/**
 * Use HTTP/1.0 protocol version.
 */
const HTTP_VERSION_1_0 = 1;

/**
 * Use HTTP/1.1 protocol version.
 */
const HTTP_VERSION_1_1 = 2;

/**
 * Attempt to use HTTP/2 protocol version. Available if libcurl is v7.33.0 or more recent and was built with nghttp2 support.
 */
const HTTP_VERSION_2_0 = 3;

/**
 * Attempt to use version 2 for HTTPS, version 1.1 for HTTP. Available if libcurl is v7.47.0 or more recent and was built with nghttp2 support.
 */
const HTTP_VERSION_2TLS = 4;

/**
 * Use any HTTP protocol version.
 */
const HTTP_VERSION_ANY = 0;

/**
 * Use TLS v1.0 encryption.
 */
const SSL_VERSION_TLSv1_0 = 4;

/**
 * Use TLS v1.1 encryption.
 */
const SSL_VERSION_TLSv1_1 = 5;

/**
 * Use TLS v1.2 encryption.
 */
const SSL_VERSION_TLSv1_2 = 6;

/**
 * Use any TLS v1 encryption.
 */
const SSL_VERSION_TLSv1 = 1;

/**
 * Use SSL v2 encryption.
 */
const SSL_VERSION_SSLv2 = 2;

/**
 * Use SSL v3 encryption.
 */
const SSL_VERSION_SSLv3 = 3;

/**
 * Use any encryption.
 */
const SSL_VERSION_ANY = 0;

/**
 * Use TLS SRP authentication. Available if libcurl is v7.21.4 or more recent and was built with gnutls or openssl with TLS-SRP support.
 */
const TLSAUTH_SRP = 1;

/**
 * Use IPv4 resolver.
 */
const IPRESOLVE_V4 = 1;

/**
 * Use IPv6 resolver.
 */
const IPRESOLVE_V6 = 2;

/**
 * Use any resolver.
 */
const IPRESOLVE_ANY = 0;

/**
 * Use Basic authentication.
 */
const AUTH_BASIC = 1;

/**
 * Use Digest authentication.
 */
const AUTH_DIGEST = 2;

/**
 * Use IE (lower v7) quirks with Digest authentication. Available if libcurl is v7.19.3 or more recent.
 */
const AUTH_DIGEST_IE = 16;

/**
 * Use NTLM authentication.
 */
const AUTH_NTLM = 8;

/**
 * Use GSS-Negotiate authentication.
 */
const AUTH_GSSNEG = 4;

/**
 * Use HTTP Netgotiate authentication (SPNEGO, RFC4559). Available if libcurl is v7.38.0 or more recent.
 */
const AUTH_SPNEGO = 4;

/**
 * Use any authentication.
 */
const AUTH_ANY = -17;

/**
 * Use SOCKSv4 proxy protocol.
 */
const PROXY_SOCKS4 = 4;

/**
 * Use SOCKSv4a proxy protocol.
 */
const PROXY_SOCKS4A = 5;

/**
 * Use SOCKS5h proxy protocol.
 */
const PROXY_SOCKS5_HOSTNAME = 5;

/**
 * Use SOCKS5 proxy protoccol.
 */
const PROXY_SOCKS5 = 5;

/**
 * Use HTTP/1.1 proxy protocol.
 */
const PROXY_HTTP = 0;

/**
 * Use HTTP/1.0 proxy protocol. Available if libcurl is v7.19.4 or more recent.
 */
const PROXY_HTTP_1_0 = 1;

/**
 * Keep POSTing on 301 redirects. Available if libcurl is v7.19.1 or more recent.
 */
const POSTREDIR_301 = 1;

/**
 * Keep POSTing on 302 redirects. Available if libcurl is v7.19.1 or more recent.
 */
const POSTREDIR_302 = 2;

/**
 * Keep POSTing on 303 redirects. Available if libcurl is v7.19.1 or more recent.
 */
const POSTREDIR_303 = 4;

/**
 * Keep POSTing on any redirect. Available if libcurl is v7.19.1 or more recent.
 */
const POSTREDIR_ALL = 7;

namespace Client;

/**
 * The Client\Request class provides an HTTP message implementation tailored to represent a request message to be sent by the client.
 *
 * See Client::enqueue().
 */
class Request extends Message
{
    /**
     * Array of options for this request, which override client options.
     *
     * @var array
     */
    protected $options = null;

    /**
     * Create a new client request message to be enqueued and sent by Client.
     *
     * @param string $meth The request method.
     * @param string $url The request URL.
     * @param array $headers HTTP headers.
     * @param Message\Body $body Request body.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     */
    public function __construct(
        string $meth = null,
        string $url = null,
        array $headers = null,
        Message\Body $body = null,
    ) {}

    /**
     * Add querystring data.
     * See Client\Request::setQuery() and Message::setRequestUrl().
     *
     * @param mixed $query_data Additional querystring data.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadQueryStringException
     * @return Client\Request self.
     */
    public function addQuery($query_data)
    {
    }

    /**
     * Add specific SSL options.
     * See Client\Request::setSslOptions(), Client\Request::setOptions() and Client\Curl\$ssl options.
     *
     * @param array $ssl_options Add this SSL options.
     * @throws Exception\InvalidArgumentException
     * @return Client\Request self.
     */
    public function addSslOptions(array $ssl_options = null)
    {
    }

    /**
     * Extract the currently set "Content-Type" header.
     * See Client\Request::setContentType().
     *
     * @return string|null string the currently set content type.
     * 		 or NULL if no "Content-Type" header is set.
     */
    public function getContentType()
    {
    }

    /**
     * Get priorly set options.
     * See Client\Request::setOptions().
     *
     * @return array options.
     */
    public function getOptions()
    {
    }

    /**
     * Retrieve the currently set querystring.
     *
     * @return string|null string the currently set querystring.
     * 		 or NULL if no querystring is set.
     */
    public function getQuery()
    {
    }

    /**
     * Retrieve priorly set SSL options.
     * See Client\Request::getOptions() and Client\Request::setSslOptions().
     *
     * @return array SSL options.
     */
    public function getSslOptions()
    {
    }

    /**
     * Set the MIME content type of the request message.
     *
     * @param string $content_type The MIME type used as "Content-Type".
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Client\Request self.
     */
    public function setContentType(string $content_type)
    {
    }

    /**
     * Set client options.
     * See Client::setOptions() and Client\Curl.
     *
     * Request specific options override general options which were set in the client.
     *
     * ***NOTE:***
     * Only options specified prior enqueueing a request are applied to the request.
     *
     * @param array $options The options to set.
     * @throws Exception\InvalidArgumentException
     * @return Client\Request self.
     */
    public function setOptions(array $options = null)
    {
    }

    /**
     * (Re)set the querystring.
     * See Client\Request::addQuery() and Message::setRequestUrl().
     *
     * @param mixed $query_data
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadQueryStringException
     * @return Client\Request self.
     */
    public function setQuery($query_data)
    {
    }

    /**
     * Specifically set SSL options.
     * See Client\Request::setOptions() and Client\Curl\$ssl options.
     *
     * @param array $ssl_options Set SSL options to this array.
     * @throws Exception\InvalidArgumentException
     * @return Client\Request self.
     */
    public function setSslOptions(array $ssl_options = null)
    {
    }
}

/**
 * The Client\Response class represents an HTTP message the client returns as answer from a server to an Client\Request.
 */
class Response extends Message
{
    /**
     * Extract response cookies.
     * Parses any "Set-Cookie" response headers into an Cookie list. See Cookie::__construct().
     *
     * @param int $flags Cookie parser flags.
     * @param array $allowed_extras List of keys treated as extras.
     * @return array list of Cookie instances.
     */
    public function getCookies(int $flags = 0, array $allowed_extras = null)
    {
    }

    /**
     * Retrieve transfer related information after the request has completed.
     * See Client::getTransferInfo().
     *
     * @param string $name A key to retrieve out of the transfer info.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\BadMethodCallException
     * @throws Exception\UnexpectedValueException
     * @return object|mixed object stdClass instance with all transfer info if $name was not given.
     * 		 or mixed the specific transfer info for $name.
     */
    public function getTransferInfo(string $name = null)
    {
    }
}

namespace Client\Curl;

/**
 * Interface to an user event loop implementation for Client::configure()'s $use_eventloop option.
 *
 * ***NOTE:***
 * This interface was added in v2.6.0, resp. v3.1.0.
 */
interface User
{
    /**
     * No action.
     */
    public const POLL_NONE = 0;

    /**
     * Poll for read readiness.
     */
    public const POLL_IN = 1;

    /**
     * Poll for write readiness.
     */
    public const POLL_OUT = 2;

    /**
     * Poll for read/write readiness.
     */
    public const POLL_INOUT = 3;

    /**
     * Stop polling for activity on this descriptor.
     */
    public const POLL_REMOVE = 4;

    /**
     * Initialize the event loop.
     *
     * @param callable $run as function(http\Client $c, resource $s = null, int $action = Client\Curl\User::POLL_NONE) : int
     *   Internal callback returning the number of unfinished requests pending.
     *
     *
     * ***NOTE***:
     * The callback should be run when a timeout occurs or a watched socket needs action.
     */
    public function init(callable $run);

    /**
     * Run the loop as long as it does not block.
     *
     * ***NOTE:***
     * This method is called by Client::once(), so it does not need to have an actual implementation if Client::once() is never called.
     */
    public function once();

    /**
     * Run the loop.
     *
     * ***NOTE:***
     * This method is called by Client::send(), so it does not need to have an actual implementation if Client::send() is never called.
     */
    public function send();

    /**
     * Register (or deregister) a socket watcher.
     *
     * @param resource $socket The socket descriptor to watch.
     * @param int $poll Client\Curl\User::POLL_* constant.
     */
    public function socket($socket, int $poll);

    /**
     * Register a timeout watcher.
     *
     * @param int $timeout_ms Desired maximum timeout in milliseconds.
     */
    public function timer(int $timeout_ms);

    /**
     * Wait/poll/select (block the loop) until events fire.
     *
     * ***NOTE:***
     * This method is called by Client::wait(), so it does not need to have an actual implementation if Client::wait() is never called.
     *
     * @param int $timeout_ms Block for at most this milliseconds.
     */
    public function wait(int $timeout_ms = null);
}

/**
 * CURL feature constants.
 *
 * ***NOTE:***
 * These constants have been added in v2.6.0, resp. v3.1.0.
 */

namespace Client\Curl\Features;

/**
 * Whether libcurl supports asynchronous domain name resolution.
 */
const ASYNCHDNS = 128;

/**
 * Whether libcurl supports the Generic Security Services Application Program Interface. Available if libcurl is v7.38.0 or more recent.
 */
const GSSAPI = 131072;

/**
 * Whether libcurl supports HTTP Generic Security Services negotiation.
 */
const GSSNEGOTIATE = 32;

/**
 * Whether libcurl supports the HTTP/2 protocol. Available if libcurl is v7.33.0 or more recent.
 */
const HTTP2 = 65536;

/**
 * Whether libcurl supports international domain names.
 */
const IDN = 1024;

/**
 * Whether libcurl supports IPv6.
 */
const IPV6 = 1;

/**
 * Whether libcurl supports the old Kerberos protocol.
 */
const KERBEROS4 = 2;

/**
 * Whether libcurl supports the more recent Kerberos v5 protocol. Available if libcurl is v7.40.0 or more recent.
 */
const KERBEROS5 = 262144;

/**
 * Whether libcurl supports large files.
 */
const LARGEFILE = 512;

/**
 * Whether libcurl supports gzip/deflate compression.
 */
const LIBZ = 8;

/**
 * Whether libcurl supports the NT Lan Manager authentication.
 */
const NTLM = 16;

/**
 * Whether libcurl supports NTLM delegation to a winbind helper. Available if libcurl is v7.22.0 or more recent.
 */
const NTLM_WB = 32768;

/**
 * Whether libcurl supports the Public Suffix List for cookie host handling. Available if libcurl is v7.47.0 or more recent.
 */
const PSL = 1048576;

/**
 * Whether libcurl supports the Simple and Protected GSSAPI Negotiation Mechanism.
 */
const SPNEGO = 256;

/**
 * Whether libcurl supports SSL/TLS protocols.
 */
const SSL = 4;

/**
 * Whether libcurl supports the Security Support Provider Interface.
 */
const SSPI = 2048;

/**
 * Whether libcurl supports TLS Secure Remote Password authentication. Available if libcurl is v7.21.4 or more recent.
 */
const TLSAUTH_SRP = 16384;

/**
 * Whether libcurl supports connections to unix sockets. Available if libcurl is v7.40.0 or more recent.
 */
const UNIX_SOCKETS = 524288;

/**
 * CURL version constants.
 *
 * ***NOTE:***
 * These constants have been added in v2.6.0, resp. v3.1.0.
 */

namespace Client\Curl\Versions;

/**
 * Version string of libcurl, e.g. "7.50.0".
 */
const CURL = '7.64.0';

/**
 * Version string of the SSL/TLS library, e.g. "OpenSSL/1.0.2h".
 */
const SSL = 'OpenSSL/1.1.1b';

/**
 * Version string of the zlib compression library, e.g. "1.2.8".
 */
const LIBZ = '1.2.11';

/**
 * Version string of the c-ares library, e.g. "1.11.0".
 */
const ARES = null;

/**
 * Version string of the IDN library, e.g. "1.32".
 */
const IDN = null;

namespace Encoding;

/**
 * Base class for encoding stream implementations.
 */
abstract class Stream
{
    /**
     * Do no intermittent flushes.
     */
    public const FLUSH_NONE = 0;

    /**
     * Flush at appropriate transfer points.
     */
    public const FLUSH_SYNC = 1048576;

    /**
     * Flush at each IO operation.
     */
    public const FLUSH_FULL = 2097152;

    /**
     * Base constructor for encoding stream implementations.
     *
     * @param int $flags See Encoding\Stream and implementation specific constants.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     */
    public function __construct(int $flags = 0) {}

    /**
     * Check whether the encoding stream is already done.
     *
     * @return bool whether the encoding stream is completed.
     */
    public function done()
    {
    }

    /**
     * Finish and reset the encoding stream.
     * Returns any pending data.
     *
     * @return string any pending data.
     */
    public function finish()
    {
    }

    /**
     * Flush the encoding stream.
     * Returns any pending data.
     *
     * @return string any pending data.
     */
    public function flush()
    {
    }

    /**
     * Update the encoding stream with more input.
     *
     * @param string $data The data to pass through the stream.
     * @return string processed data.
     */
    public function update(string $data)
    {
    }
}

namespace Encoding\Stream;

/**
 * A [brotli](https://brotli.org) decoding stream.
 *
 * ***NOTE:***
 * This class has been added in v3.2.0.
 */
class Debrotli extends Encoding\Stream
{
    /**
     * Decode brotli encoded data.
     *
     * @param string $data The data to uncompress.
     * @return string the uncompressed data.
     */
    public function decode(string $data)
    {
    }
}

/**
 * A stream decoding data encoded with chunked transfer encoding.
 */
class Dechunk extends Encoding\Stream
{
    /**
     * Decode chunked encoded data.
     *
     * @param string $data The data to decode.
     * @param int &$decoded_len Out parameter with the length of $data that's been decoded.
     *   Should be ```strlen($data)``` if not truncated.
     * @return string|string|string|false string the decoded data.
     * 		 or string the unencoded data.
     * 		 or string the truncated decoded data.
     * 		 or false if $data cannot be decoded.
     */
    public function decode(string $data, int &$decoded_len = 0)
    {
    }
}

/**
 * A deflate stream supporting deflate, zlib and gzip encodings.
 */
class Deflate extends Encoding\Stream
{
    /**
     * Gzip encoding. RFC1952
     */
    public const TYPE_GZIP = 16;

    /**
     * Zlib encoding. RFC1950
     */
    public const TYPE_ZLIB = 0;

    /**
     * Deflate encoding. RFC1951
     */
    public const TYPE_RAW = 32;

    /**
     * Default compression level.
     */
    public const LEVEL_DEF = 0;

    /**
     * Least compression level.
     */
    public const LEVEL_MIN = 1;

    /**
     * Greatest compression level.
     */
    public const LEVEL_MAX = 9;

    /**
     * Default compression strategy.
     */
    public const STRATEGY_DEF = 0;

    /**
     * Filtered compression strategy.
     */
    public const STRATEGY_FILT = 256;

    /**
     * Huffman strategy only.
     */
    public const STRATEGY_HUFF = 512;

    /**
     * Run-length encoding strategy.
     */
    public const STRATEGY_RLE = 768;

    /**
     * Encoding with fixed Huffman codes only.
     *
     * > **A note on the compression strategy:**
     * >
     * > The strategy parameter is used to tune the compression algorithm.
     * >
     * > Use the value DEFAULT_STRATEGY for normal data, FILTERED for data produced by a filter (or predictor), HUFFMAN_ONLY to force Huffman encoding only (no string match), or RLE to limit match distances to one (run-length encoding).
     * >
     * > Filtered data consists mostly of small values with a somewhat random distribution. In this case, the compression algorithm is tuned to compress them better. The effect of FILTERED is to force more Huffman coding and less string matching; it is somewhat intermediate between DEFAULT_STRATEGY and HUFFMAN_ONLY.
     * >
     * > RLE is designed to be almost as fast as HUFFMAN_ONLY, but give better compression for PNG image data.
     * >
     * > FIXED prevents the use of dynamic Huffman codes, allowing for a simpler decoder for special applications.
     * >
     * > The strategy parameter only affects the compression ratio but not the correctness of the compressed output even if it is not set appropriately.
     * >
     * >_Source: [zlib Manual](http://www.zlib.net/manual.html)_
     */
    public const STRATEGY_FIXED = 1024;

    /**
     * Encode data with deflate/zlib/gzip encoding.
     *
     * @param string $data The data to compress.
     * @param int $flags Any compression tuning flags. See Encoding\Stream\Deflate and Encoding\Stream constants.
     * @return string the compressed data.
     */
    public function encode(string $data, int $flags = 0)
    {
    }
}

/**
 * A [brotli](https://brotli.org) encoding stream.
 *
 * ***NOTE:***
 * This class has been added in v3.2.0.
 */
class Enbrotli extends Encoding\Stream
{
    /**
     * Default compression level.
     */
    public const LEVEL_DEF = null;

    /**
     * Least compression level.
     */
    public const LEVEL_MIN = null;

    /**
     * Greatest compression level.
     */
    public const LEVEL_MAX = null;

    /**
     * Default window bits.
     */
    public const WBITS_DEF = null;

    /**
     * Minimum window bits.
     */
    public const WBITS_MIN = null;

    /**
     * Maximum window bits.
     */
    public const WBITS_MAX = null;

    /**
     * Default compression mode.
     */
    public const MODE_GENERIC = null;

    /**
     * Compression mode for UTF-8 formatted text.
     */
    public const MODE_TEXT = null;

    /**
     * Compression mode used in WOFF 2.0.
     */
    public const MODE_FONT = null;

    /**
     * Encode data with brotli encoding.
     *
     * @param string $data The data to compress.
     * @param int $flags Any compression tuning flags. See Encoding\Stream\Enbrotli and Encoding\Stream constants.
     * @return string the compressed data.
     */
    public function encode(string $data, int $flags = 0)
    {
    }
}

/**
 * A inflate stream supporting deflate, zlib and gzip encodings.
 */
class Inflate extends Encoding\Stream
{
    /**
     * Decode deflate/zlib/gzip encoded data.
     *
     * @param string $data The data to uncompress.
     * @return string the uncompressed data.
     */
    public function decode(string $data)
    {
    }
}

namespace Env;

/**
 * The Env\Request class' instances represent the server's current HTTP request.
 *
 * See Message for inherited members.
 */
class Request extends Message
{
    /**
     * The request's query parameters. ($_GET)
     *
     * @var QueryString
     */
    protected $query = null;

    /**
     * The request's form parameters. ($_POST)
     *
     * @var QueryString
     */
    protected $form = null;

    /**
     * The request's form uploads. ($_FILES)
     *
     * @var array
     */
    protected $files = null;

    /**
     * The request's cookies. ($_COOKIE)
     *
     * @var array
     */
    protected $cookie = null;

    /**
     * Create an instance of the server's current HTTP request.
     *
     * Upon construction, the Env\Request acquires QueryString instances of query parameters ($\_GET) and form parameters ($\_POST).
     *
     * It also compiles an array of uploaded files ($\_FILES) more comprehensive than the original $\_FILES array, see Env\Request::getFiles() for that matter.
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     */
    public function __construct() {}

    /**
     * Retrieve an URL query value ($_GET).
     *
     * See QueryString::get() and QueryString::TYPE_* constants.
     *
     * @param string $name The key to retrieve the value for.
     * @param mixed $type The type to cast the value to. See QueryString::TYPE_* constants.
     * @param mixed $defval The default value to return if the key $name does not exist.
     * @param bool $delete Whether to delete the entry from the querystring after retrieval.
     * @return QueryString|string|mixed|mixed|string QueryString if called without arguments.
     * 		 or string the whole querystring if $name is of zero length.
     * 		 or mixed $defval if the key $name does not exist.
     * 		 or mixed the querystring value cast to $type if $type was specified and the key $name exists.
     * 		 or string the querystring value if the key $name exists and $type is not specified or equals QueryString::TYPE_STRING.
     */
    public function getCookie(string $name = null, $type = null, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve the uploaded files list ($_FILES).
     *
     * @return array the consolidated upload files array.
     */
    public function getFiles()
    {
    }

    /**
     * Retrieve a form value ($_POST).
     *
     * See QueryString::get() and QueryString::TYPE_* constants.
     *
     * @param string $name The key to retrieve the value for.
     * @param mixed $type The type to cast the value to. See QueryString::TYPE_* constants.
     * @param mixed $defval The default value to return if the key $name does not exist.
     * @param bool $delete Whether to delete the entry from the querystring after retrieval.
     * @return QueryString|string|mixed|mixed|string QueryString if called without arguments.
     * 		 or string the whole querystring if $name is of zero length.
     * 		 or mixed $defval if the key $name does not exist.
     * 		 or mixed the querystring value cast to $type if $type was specified and the key $name exists.
     * 		 or string the querystring value if the key $name exists and $type is not specified or equals QueryString::TYPE_STRING.
     */
    public function getForm(string $name = null, $type = null, $defval = null, bool $delete = false)
    {
    }

    /**
     * Retrieve an URL query value ($_GET).
     *
     * See QueryString::get() and QueryString::TYPE_* constants.
     *
     * @param string $name The key to retrieve the value for.
     * @param mixed $type The type to cast the value to. See QueryString::TYPE_* constants.
     * @param mixed $defval The default value to return if the key $name does not exist.
     * @param bool $delete Whether to delete the entry from the querystring after retrieval.
     * @return QueryString|string|mixed|mixed|string QueryString if called without arguments.
     * 		 or string the whole querystring if $name is of zero length.
     * 		 or mixed $defval if the key $name does not exist.
     * 		 or mixed the querystring value cast to $type if $type was specified and the key $name exists.
     * 		 or string the querystring value if the key $name exists and $type is not specified or equals QueryString::TYPE_STRING.
     */
    public function getQuery(string $name = null, $type = null, $defval = null, bool $delete = false)
    {
    }
}

/**
 * The Env\Response class' instances represent the server's current HTTP response.
 *
 * See Message for inherited members.
 */
class Response extends Message
{
    /**
     * Do not use content encoding.
     */
    public const CONTENT_ENCODING_NONE = 0;

    /**
     * Support "Accept-Encoding" requests with gzip and deflate encoding.
     */
    public const CONTENT_ENCODING_GZIP = 1;

    /**
     * No caching info available.
     */
    public const CACHE_NO = 0;

    /**
     * The cache was hit.
     */
    public const CACHE_HIT = 1;

    /**
     * The cache was missed.
     */
    public const CACHE_MISS = 2;

    /**
     * A request instance which overrides the environments default request.
     *
     * @var Env\Request
     */
    protected $request = null;

    /**
     * The response's MIME content type.
     *
     * @var string
     */
    protected $contentType = null;

    /**
     * The response's MIME content disposition.
     *
     * @var string
     */
    protected $contentDisposition = null;

    /**
     * See Env\Response::CONTENT_ENCODING_* constants.
     *
     * @var int
     */
    protected $contentEncoding = null;

    /**
     * How the client should treat this response in regards to caching.
     *
     * @var string
     */
    protected $cacheControl = null;

    /**
     * A custom ETag.
     *
     * @var string
     */
    protected $etag = null;

    /**
     * A "Last-Modified" time stamp.
     *
     * @var int
     */
    protected $lastModified = null;

    /**
     * Any throttling delay.
     *
     * @var int
     */
    protected $throttleDelay = null;

    /**
     * The chunk to send every $throttleDelay seconds.
     *
     * @var int
     */
    protected $throttleChunk = null;

    /**
     * The response's cookies.
     *
     * @var array
     */
    protected $cookies = null;

    /**
     * Create a new env response message instance.
     *
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     */
    public function __construct() {}

    /**
     * Output buffer handler.
     * Appends output data to the body.
     *
     * @param string $data The data output.
     * @param int $ob_flags Output buffering flags passed from the output buffering control layer.
     * @return bool
     */
    public function __invoke(string $data, int $ob_flags = 0)
    {
    }

    /**
     * Manually test the header $header_name of the environment's request for a cache hit.
     * Env\Response::send() checks that itself, though.
     *
     * @param string $header_name The request header to test.
     * @return int a Env\Response::CACHE_* constant.
     */
    public function isCachedByEtag(string $header_name = 'If-None-Match')
    {
    }

    /**
     * Manually test the header $header_name of the environment's request for a cache hit.
     * Env\Response::send() checks that itself, though.
     *
     * @param string $header_name The request header to test.
     * @return int a Env\Response::CACHE_* constant.
     */
    public function isCachedByLastModified(string $header_name = 'If-Modified-Since')
    {
    }

    /**
     * Send the response through the SAPI or $stream.
     * Flushes all output buffers.
     *
     * @param resource $stream A writable stream to send the response through.
     * @return bool
     */
    public function send($stream = null)
    {
    }

    /**
     * Make suggestions to the client how it should cache the response.
     *
     * @param string $cache_control (A) "Cache-Control" header value(s).
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setCacheControl(string $cache_control)
    {
    }

    /**
     * Set the response's content disposition parameters.
     *
     * @param array $disposition_params MIME content disposition as Params array.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setContentDisposition(array $disposition_params)
    {
    }

    /**
     * Enable support for "Accept-Encoding" requests with deflate or gzip.
     * The response will be compressed if the client indicates support and wishes that.
     *
     * @param int $content_encoding See Env\Response::CONTENT_ENCODING_* constants.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setContentEncoding(int $content_encoding)
    {
    }

    /**
     * Set the MIME content type of the response.
     *
     * @param string $content_type The response's content type.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setContentType(string $content_type)
    {
    }

    /**
     * Add cookies to the response to send.
     *
     * @param mixed $cookie The cookie to send.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return Env\Response self.
     */
    public function setCookie($cookie)
    {
    }

    /**
     * Override the environment's request.
     *
     * @param Message $env_request The overriding request message.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setEnvRequest(http\Message $env_request)
    {
    }

    /**
     * Set a custom ETag.
     *
     * ***NOTE:***
     * This will be used for caching and pre-condition checks.
     *
     * @param string $etag A ETag.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setEtag(string $etag)
    {
    }

    /**
     * Set a custom last modified time stamp.
     *
     * ***NOTE:***
     * This will be used for caching and pre-condition checks.
     *
     * @param int $last_modified A unix timestamp.
     * @throws Exception\InvalidArgumentException
     * @return Env\Response self.
     */
    public function setLastModified(int $last_modified)
    {
    }

    /**
     * Enable throttling.
     * Send $chunk_size bytes every $delay seconds.
     *
     * ***NOTE:***
     * If you need throttling by regular means, check for other options in your stack, because this method blocks the executing process/thread until the response has completely been sent.
     *
     * @param int $chunk_size Bytes to send.
     * @param float $delay Seconds to sleep.
     * @return Env\Response self.
     */
    public function setThrottleRate(int $chunk_size, float $delay = 1)
    {
    }
}

/**
 * URL class using the HTTP environment by default.
 *
 * ***NOTE:***
 * This class has been added in v3.0.0.
 *
 * Always adds Url::FROM_ENV to the $flags constructor argument. See also Url.
 */
class Url extends Url
{
}

namespace Exception;

/**
 * A bad conversion (e.g. character conversion) was encountered.
 */
class BadConversionException extends \DomainException implements Exception
{
}

/**
 * A bad HTTP header was encountered.
 */
class BadHeaderException extends \DomainException implements Exception
{
}

/**
 * A bad HTTP message was encountered.
 */
class BadMessageException extends \DomainException implements Exception
{
}

/**
 * A method was called on an object, which was in an invalid or unexpected state.
 */
class BadMethodCallException extends \BadMethodCallException implements Exception
{
}

/**
 * A bad querystring was encountered.
 */
class BadQueryStringException extends \DomainException implements Exception
{
}

/**
 * A bad HTTP URL was encountered.
 */
class BadUrlException extends \DomainException implements Exception
{
}

/**
 * One or more invalid arguments were passed to a method.
 */
class InvalidArgumentException extends \InvalidArgumentException implements Exception
{
}

/**
 * A generic runtime exception.
 */
class RuntimeException extends \RuntimeException implements Exception
{
}

/**
 * An unexpected value was encountered.
 */
class UnexpectedValueException extends \UnexpectedValueException implements Exception
{
}

namespace Header;

/**
 * The parser which is underlying Header and Message.
 *
 * ***NOTE:***
 * This class has been added in v2.3.0.
 */
class Parser
{
    /**
     * Finish up parser at end of (incomplete) input.
     */
    public const CLEANUP = 1;

    /**
     * Parse failure.
     */
    public const STATE_FAILURE = -1;

    /**
     * Expecting HTTP info (request/response line) or headers.
     */
    public const STATE_START = 0;

    /**
     * Expecting a key or already parsing a key.
     */
    public const STATE_KEY = 1;

    /**
     * Expecting a value or already parsing the value.
     */
    public const STATE_VALUE = 2;

    /**
     * At EOL of an header, checking whether a folded header line follows.
     */
    public const STATE_VALUE_EX = 3;

    /**
     * A header was completed.
     */
    public const STATE_HEADER_DONE = 4;

    /**
     * Finished parsing the headers.
     *
     * ***NOTE:***
     * Most of this states won't be returned to the user, because the parser immediately jumps to the next expected state.
     */
    public const STATE_DONE = 5;

    /**
     * Retrieve the current state of the parser.
     * See Header\Parser::STATE_* constants.
     *
     * @throws Exception\InvalidArgumentException
     * @return int Header\Parser::STATE_* constant.
     */
    public function getState()
    {
    }

    /**
     * Parse a string.
     *
     * @param string $data The (part of the) header to parse.
     * @param int $flags Any combination of [parser flags](http/Header/Parser#Parser.flags:).
     * @param array &$header Successfully parsed headers.
     * @throws Exception\InvalidArgumentException
     * @return int Header\Parser::STATE_* constant.
     */
    public function parse(string $data, int $flags, array &$header = null)
    {
    }

    /**
     * Parse a stream.
     *
     * @param resource $stream The header stream to parse from.
     * @param int $flags Any combination of [parser flags](http/Header/Parser#Parser.flags:).
     * @param array &$headers The headers parsed.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return int Header\Parser::STATE_* constant.
     */
    public function stream($stream, int $flags, array &$headers)
    {
    }
}

namespace Message;

/**
 * The message body, represented as a PHP (temporary) stream.
 *
 * ***NOTE:***
 * Currently, Message\Body::addForm() creates multipart/form-data bodies.
 */
class Body implements \Serializable
{
    /**
     * Create a new message body, optionally referencing $stream.
     *
     * @param resource $stream A stream to be used as message body.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     */
    public function __construct($stream = null) {}

    /**
     * String cast handler.
     *
     * @return string the message body.
     */
    public function __toString()
    {
    }

    /**
     * Add form fields and files to the message body.
     *
     * ***NOTE:***
     * Currently, Message\Body::addForm() creates "multipart/form-data" bodies.
     *
     * @param array $fields List of form fields to add.
     * @param array $files List of form files to add.
     *
     * $fields must look like:
     *
     *     [
     *       "field_name" => "value",
     *       "multi_field" => [
     *         "value1",
     *         "value2"
     *       ]
     *     ]
     *
     * $files must look like:
     *
     *     [
     *       [
     *         "name" => "field_name",
     *         "type" => "content/type",
     *         "file" => "/path/to/file.ext"
     *       ], [
     *         "name" => "field_name2",
     *         "type" => "text/plain",
     *         "file" => "file.ext",
     *         "data" => "string"
     *       ], [
     *         "name" => "field_name3",
     *         "type" => "image/jpeg",
     *         "file" => "file.ext",
     *         "data" => fopen("/home/mike/Pictures/mike.jpg","r")
     *     ]
     *
     * As you can see, a file structure must contain a "file" entry, which holds a file path, and an optional "data" entry, which may either contain a resource to read from or the actual data as string.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     * @return Message\Body self.
     */
    public function addForm(array $fields = null, array $files = null)
    {
    }

    /**
     * Add a part to a multipart body.
     *
     * @param Message $part The message part.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     * @return Message\Body self.
     */
    public function addPart(http\Message $part)
    {
    }

    /**
     * Append plain bytes to the message body.
     *
     * @param string $data The data to append to the body.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\RuntimeException
     * @return Message\Body self.
     */
    public function append(string $data)
    {
    }

    /**
     * Retrieve the ETag of the body.
     *
     * @return string|string|false string an Apache style ETag of inode, mtime and size in hex concatenated by hyphens if the message body stream is stat-able.
     * 		 or string a content hash (which algorithm is determined by INI http.etag.mode) if the stream is not stat-able.
     * 		 or false if http.etag.mode is not a known hash algorithm.
     */
    public function etag()
    {
    }

    /**
     * Retrieve any boundary of the message body.
     * See Message::splitMultipartBody().
     *
     * @return string|null string the message body boundary.
     * 		 or NULL if this message body has no boundary.
     */
    public function getBoundary()
    {
    }

    /**
     * Retrieve the underlying stream resource.
     *
     * @return resource the underlying stream.
     */
    public function getResource()
    {
    }

    /**
     * Implements Serializable.
     * Alias of Message\Body::__toString().
     *
     * @return string serialized message body.
     */
    public function serialize()
    {
    }

    /**
     * Stat size, atime, mtime and/or ctime.
     *
     * @param string $field A single stat field to retrieve.
     * @return int|object int the requested stat field.
     * 		 or object stdClass instance holding all four stat fields.
     */
    public function stat(string $field = null)
    {
    }

    /**
     * Stream the message body through a callback.
     *
     * @param callable $callback The callback of the form function(http\Message\Body $from, string $data).
     * @param int $offset Start to stream from this offset.
     * @param int $maxlen Stream at most $maxlen bytes, or all if $maxlen is less than 1.
     * @return Message\Body self.
     */
    public function toCallback(callable $callback, int $offset = 0, int $maxlen = 0)
    {
    }

    /**
     * Stream the message body into another stream $stream, starting from $offset, streaming $maxlen at most.
     *
     * @param resource $stream The resource to write to.
     * @param int $offset The starting offset.
     * @param int $maxlen The maximum amount of data to stream. All content if less than 1.
     * @return Message\Body self.
     */
    public function toStream($stream, int $offset = 0, int $maxlen = 0)
    {
    }

    /**
     * Retrieve the message body serialized to a string.
     * Alias of Message\Body::__toString().
     *
     * @return string message body.
     */
    public function toString()
    {
    }

    /**
     * Implements Serializable.
     *
     * @param string $serialized The serialized message body.
     */
    public function unserialize($serialized)
    {
    }
}

/**
 * The parser which is underlying Message.
 *
 * ***NOTE:***
 * This class was added in v2.2.0.
 */
class Parser
{
    /**
     * Finish up parser at end of (incomplete) input.
     */
    public const CLEANUP = 1;

    /**
     * Soak up the rest of input if no entity length is deducible.
     */
    public const DUMB_BODIES = 2;

    /**
     * Redirect messages do not contain any body despite of indication of such.
     */
    public const EMPTY_REDIRECTS = 4;

    /**
     * Continue parsing while input is available.
     */
    public const GREEDY = 8;

    /**
     * Parse failure.
     */
    public const STATE_FAILURE = -1;

    /**
     * Expecting HTTP info (request/response line) or headers.
     */
    public const STATE_START = 0;

    /**
     * Parsing headers.
     */
    public const STATE_HEADER = 1;

    /**
     * Completed parsing headers.
     */
    public const STATE_HEADER_DONE = 2;

    /**
     * Parsing the body.
     */
    public const STATE_BODY = 3;

    /**
     * Soaking up all input as body.
     */
    public const STATE_BODY_DUMB = 4;

    /**
     * Reading body as indicated by `Content-Length` or `Content-Range`.
     */
    public const STATE_BODY_LENGTH = 5;

    /**
     * Parsing `chunked` encoded body.
     */
    public const STATE_BODY_CHUNKED = 6;

    /**
     * Finished parsing the body.
     */
    public const STATE_BODY_DONE = 7;

    /**
     * Updating Content-Length based on body size.
     */
    public const STATE_UPDATE_CL = 8;

    /**
     * Finished parsing the message.
     *
     * ***NOTE:***
     * Most of this states won't be returned to the user, because the parser immediately jumps to the next expected state.
     */
    public const STATE_DONE = 9;

    /**
     * Retrieve the current state of the parser.
     * See Message\Parser::STATE_* constants.
     *
     * @throws Exception\InvalidArgumentException
     * @return int Message\Parser::STATE_* constant.
     */
    public function getState()
    {
    }

    /**
     * Parse a string.
     *
     * @param string $data The (part of the) message to parse.
     * @param int $flags Any combination of [parser flags](http/Message/Parser#Parser.flags:).
     * @param Message $message The current state of the message parsed.
     * @throws Exception\InvalidArgumentException
     * @return int Message\Parser::STATE_* constant.
     */
    public function parse(string $data, int $flags, Message $message)
    {
    }

    /**
     * Parse a stream.
     *
     * @param resource $stream The message stream to parse from.
     * @param int $flags Any combination of [parser flags](http/Message/Parser#Parser.flags:).
     * @param Message $message The current state of the message parsed.
     * @throws Exception\InvalidArgumentException
     * @throws Exception\UnexpectedValueException
     * @return int
     */
    public function stream($stream, int $flags, Message $message)
    {
    }
}
