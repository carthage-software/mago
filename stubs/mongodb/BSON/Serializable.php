<?php

namespace MongoDB\BSON;

use stdClass;

/**
 * Classes that implement this interface may return data to be serialized as a BSON array or document in lieu of the object's public properties
 * @link https://php.net/manual/en/class.mongodb-bson-serializable.php
 */
interface Serializable extends Type
{
    /**
     * Provides an array or document to serialize as BSON
     * Called during serialization of the object to BSON. The method must return an array or stdClass.
     * Root documents (e.g. a MongoDB\BSON\Serializable passed to MongoDB\BSON\fromPHP()) will always be serialized as a BSON document.
     * For field values, associative arrays and stdClass instances will be serialized as a BSON document and sequential arrays (i.e. sequential, numeric indexes starting at 0) will be serialized as a BSON array.
     * @link https://php.net/manual/en/mongodb-bson-serializable.bsonserialize.php
     */
    public function bsonSerialize(): array|stdClass|Document|PackedArray;
}
