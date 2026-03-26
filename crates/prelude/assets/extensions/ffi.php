<?php

namespace {
    use FFI\CData;
    use FFI\CType;
    use FFI\ParserException;

    class FFI
    {
        /**
         * @param string $code
         * @param string|null $lib
         *
         * @throws ParserException
         */
        public static function cdef(string $code = '', ?string $lib = null): FFI {}

        /**
         * @param string $filename
         *
         * @return FFI|null
         */
        public static function load(string $filename): ?FFI {}

        public static function scope(string $name): FFI {}

        /**
         * @param string|CType $type
         *
         * @throws ParserException
         */
        public static function new($type, bool $owned = true, bool $persistent = false): ?CData {}

        public static function free(CData $ptr): void {}

        /**
         * @param CType|string $type
         * @param CData|int|float|bool|null $ptr
         */
        public static function cast($type, $ptr): ?CData {}

        public static function type(string $type): ?CType {}

        public static function typeof(CData $ptr): CType {}

        /**
         * @param list<int> $dimensions
         */
        public static function arrayType(CType $type, array $dimensions): CType {}

        public static function addr(CData $ptr): CData {}

        /**
         * @param CData|CType $ptr
         */
        public static function sizeof($ptr): int {}

        /**
         * @param CData|CType $ptr
         */
        public static function alignof($ptr): int {}

        /**
         * @param CData|string $from
         */
        public static function memcpy(CData $to, $from, int $size): void {}

        /**
         * @param CData|string $ptr1
         * @param CData|string $ptr2
         */
        public static function memcmp($ptr1, $ptr2, int $size): int {}

        public static function memset(CData $ptr, int $value, int $size): void {}

        public static function string(CData $ptr, ?int $size = null): string {}

        public static function isNull(CData $ptr): bool {}
    }
}

namespace FFI {
    use Error;

    class Exception extends Error {}

    class ParserException extends Exception {}

    final class CData {}

    final class CType
    {
        public const TYPE_VOID = 0;

        public const TYPE_FLOAT = 1;

        public const TYPE_DOUBLE = 2;

        public const TYPE_LONGDOUBLE = 3;

        public const TYPE_UINT8 = 4;

        public const TYPE_SINT8 = 5;

        public const TYPE_UINT16 = 6;

        public const TYPE_SINT16 = 7;

        public const TYPE_UINT32 = 8;

        public const TYPE_SINT32 = 9;

        public const TYPE_UINT64 = 10;

        public const TYPE_SINT64 = 11;

        public const TYPE_ENUM = 12;

        public const TYPE_BOOL = 13;

        public const TYPE_CHAR = 14;

        public const TYPE_POINTER = 15;

        public const TYPE_FUNC = 16;

        public const TYPE_ARRAY = 17;

        public const TYPE_STRUCT = 18;

        public const ATTR_CONST = 1;

        public const ATTR_INCOMPLETE_TAG = 2;

        public const ATTR_VARIADIC = 4;

        public const ATTR_INCOMPLETE_ARRAY = 8;

        public const ATTR_VLA = 16;

        public const ATTR_UNION = 32;

        public const ATTR_PACKED = 64;

        public const ATTR_MS_STRUCT = 128;

        public const ATTR_GCC_STRUCT = 256;

        public const ABI_DEFAULT = 0;

        public const ABI_CDECL = 1;

        public const ABI_FASTCALL = 2;

        public const ABI_THISCALL = 3;

        public const ABI_STDCALL = 4;

        public const ABI_PASCAL = 5;

        public const ABI_REGISTER = 6;

        public const ABI_MS = 7;

        public const ABI_SYSV = 8;

        public const ABI_VECTORCALL = 9;

        public function getName(): string {}

        public function getKind(): int {}

        public function getSize(): int {}

        public function getAlignment(): int {}

        public function getAttributes(): int {}

        /**
         * @throws Exception
         */
        public function getEnumKind(): int {}

        /**
         * @throws Exception
         */
        public function getArrayElementType(): CType {}

        /**
         * @throws Exception
         */
        public function getArrayLength(): int {}

        /**
         * @throws Exception
         */
        public function getPointerType(): CType {}

        /**
         * @return list<string>
         *
         * @throws Exception
         */
        public function getStructFieldNames(): array {}

        /**
         * @throws Exception
         */
        public function getStructFieldOffset(string $name): int {}

        /**
         * @throws Exception
         */
        public function getStructFieldType(string $name): CType {}

        /**
         * @throws Exception
         */
        public function getFuncABI(): int {}

        /**
         * @throws Exception
         */
        public function getFuncReturnType(): CType {}

        /**
         * @throws Exception
         */
        public function getFuncParameterCount(): int {}

        /**
         * @throws Exception
         */
        public function getFuncParameterType(int $index): CType {}
    }
}
