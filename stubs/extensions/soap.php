<?php

namespace Soap {
    final class Url
    {
    }

    final class Sdl
    {
    }
}

namespace {
    class SoapClient
    {
        public function __construct(string|null $wsdl, array $options = []) {}

        /**
         * @deprecated
         */
        public function __call(string $name, array $args): mixed
        {
        }

        public function __soapCall(
            string $name,
            array $args,
            array|null $options = null,
            $inputHeaders = null,
            &$outputHeaders = null,
        ): mixed {
        }

        public function __getLastRequest(): null|string
        {
        }

        public function __getLastResponse(): null|string
        {
        }

        public function __getLastRequestHeaders(): null|string
        {
        }

        public function __getLastResponseHeaders(): null|string
        {
        }

        public function __getFunctions(): null|array
        {
        }

        public function __getTypes(): null|array
        {
        }

        public function __getCookies(): array
        {
        }

        public function __doRequest(
            string $request,
            string $location,
            string $action,
            int $version,
            bool $oneWay = false,
        ): null|string {
        }

        public function __setCookie(string $name, null|string $value): void
        {
        }

        public function __setLocation(string|null $location = null): null|string
        {
        }

        public function __setSoapHeaders($headers = null): bool
        {
        }
    }

    class SoapVar
    {
        public int $enc_type;

        public mixed $enc_value;

        public string|null $enc_stype;

        public string|null $enc_ns;

        public string|null $enc_name;

        public string|null $enc_namens;

        public function __construct(
            mixed $data,
            int|null $encoding,
            null|string $typeName,
            string|null $typeNamespace = null,
            string|null $nodeName = null,
            string|null $nodeNamespace = null,
        ) {}
    }

    class SoapServer
    {
        public function __construct(string|null $wsdl, array $options = []) {}

        public function setPersistence(int $mode): void
        {
        }

        public function setClass(string $class, mixed ...$args): void
        {
        }

        public function setObject(object $object): void
        {
        }

        public function addFunction($functions): void
        {
        }

        public function getFunctions(): array
        {
        }

        public function handle(string|null $request = null): void
        {
        }

        public function fault(
            string $code,
            string $string,
            string $actor = '',
            mixed $details = null,
            string $name = '',
        ): void {
        }

        public function addSoapHeader(SoapHeader $header): void
        {
        }

        public function __getLastResponse(): null|string
        {
        }
    }

    class SoapFault extends Exception
    {
        public null|string $faultcode;

        /**
         * @var string
         */
        public string $faultstring;

        public null|string $faultactor;

        public mixed $detail;

        public string $faultname;

        public mixed $headerfault;

        public string|null $faultcodens;

        public string|null $_name;

        public function __construct(
            array|string|null $code,
            string $string,
            string|null $actor = null,
            mixed $details = null,
            string|null $name = null,
            mixed $headerFault = null,
        ) {}

        public function __toString(): string
        {
        }
    }

    class SoapParam
    {
        public string $param_name;

        public mixed $param_data;

        public function __construct(mixed $data, string $name) {}
    }

    class SoapHeader
    {
        public string $namespace;

        public string $name;

        public mixed $data;

        public bool $mustUnderstand;

        public string|int|null $actor;

        public function __construct(
            string $namespace,
            string $name,
            mixed $data,
            bool $mustUnderstand = false,
            string|int|null $actor = null,
        ) {}
    }

    /**
     * Set whether to use the SOAP error handler
     * @link https://php.net/manual/en/function.use-soap-error-handler.php
     * @param bool $enable [optional] <p>
     * Set to <b>TRUE</b> to send error details to clients.
     * </p>
     * @return bool the original value.
     */
    function use_soap_error_handler(bool $enable = true): bool
    {
    }

    /**
     * Checks if a SOAP call has failed
     * @link https://php.net/manual/en/function.is-soap-fault.php
     * @param mixed $object <p>
     * The object to test.
     * </p>
     * @return bool This will return <b>TRUE</b> on error, and <b>FALSE</b> otherwise.
     */
    function is_soap_fault(mixed $object): bool
    {
    }

    const SOAP_1_1 = 1;

    const SOAP_1_2 = 2;

    const SOAP_PERSISTENCE_SESSION = 1;

    const SOAP_PERSISTENCE_REQUEST = 2;

    /**
     * @deprecated
     */
    const SOAP_FUNCTIONS_ALL = 999;

    const SOAP_ENCODED = 1;

    const SOAP_LITERAL = 2;

    const SOAP_RPC = 1;

    const SOAP_DOCUMENT = 2;

    const SOAP_ACTOR_NEXT = 1;

    const SOAP_ACTOR_NONE = 2;

    const SOAP_ACTOR_UNLIMATERECEIVER = 3;

    const SOAP_COMPRESSION_ACCEPT = 32;

    const SOAP_COMPRESSION_GZIP = 0;

    const SOAP_COMPRESSION_DEFLATE = 16;

    const SOAP_AUTHENTICATION_BASIC = 0;

    const SOAP_AUTHENTICATION_DIGEST = 1;

    const UNKNOWN_TYPE = 999998;

    const XSD_STRING = 101;

    const XSD_BOOLEAN = 102;

    const XSD_DECIMAL = 103;

    const XSD_FLOAT = 104;

    const XSD_DOUBLE = 105;

    const XSD_DURATION = 106;

    const XSD_DATETIME = 107;

    const XSD_TIME = 108;

    const XSD_DATE = 109;

    const XSD_GYEARMONTH = 110;

    const XSD_GYEAR = 111;

    const XSD_GMONTHDAY = 112;

    const XSD_GDAY = 113;

    const XSD_GMONTH = 114;

    const XSD_HEXBINARY = 115;

    const XSD_BASE64BINARY = 116;

    const XSD_ANYURI = 117;

    const XSD_QNAME = 118;

    const XSD_NOTATION = 119;

    const XSD_NORMALIZEDSTRING = 120;

    const XSD_TOKEN = 121;

    const XSD_LANGUAGE = 122;

    const XSD_NMTOKEN = 123;

    const XSD_NAME = 124;

    const XSD_NCNAME = 125;

    const XSD_ID = 126;

    const XSD_IDREF = 127;

    const XSD_IDREFS = 128;

    const XSD_ENTITY = 129;

    const XSD_ENTITIES = 130;

    const XSD_INTEGER = 131;

    const XSD_NONPOSITIVEINTEGER = 132;

    const XSD_NEGATIVEINTEGER = 133;

    const XSD_LONG = 134;

    const XSD_INT = 135;

    const XSD_SHORT = 136;

    const XSD_BYTE = 137;

    const XSD_NONNEGATIVEINTEGER = 138;

    const XSD_UNSIGNEDLONG = 139;

    const XSD_UNSIGNEDINT = 140;

    const XSD_UNSIGNEDSHORT = 141;

    const XSD_UNSIGNEDBYTE = 142;

    const XSD_POSITIVEINTEGER = 143;

    const XSD_NMTOKENS = 144;

    const XSD_ANYTYPE = 145;

    const XSD_ANYXML = 147;

    const APACHE_MAP = 200;

    const SOAP_ENC_OBJECT = 301;

    const SOAP_ENC_ARRAY = 300;

    const XSD_1999_TIMEINSTANT = 401;

    const XSD_NAMESPACE = 'http://www.w3.org/2001/XMLSchema';

    const XSD_1999_NAMESPACE = 'http://www.w3.org/1999/XMLSchema';

    const SOAP_SINGLE_ELEMENT_ARRAYS = 1;

    const SOAP_WAIT_ONE_WAY_CALLS = 2;

    const SOAP_USE_XSI_ARRAY_TYPE = 4;

    const WSDL_CACHE_NONE = 0;

    const WSDL_CACHE_DISK = 1;

    const WSDL_CACHE_MEMORY = 2;

    const WSDL_CACHE_BOTH = 3;

    const SOAP_SSL_METHOD_TLS = 0;

    const SOAP_SSL_METHOD_SSLv2 = 1;

    const SOAP_SSL_METHOD_SSLv3 = 2;

    const SOAP_SSL_METHOD_SSLv23 = 3;
}
