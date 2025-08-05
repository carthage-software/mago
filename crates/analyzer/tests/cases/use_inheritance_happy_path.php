<?php

trait Timestampable
{
}

class Document
{
    use Timestampable;
}

/**
 * @template T
 */
trait HasOwner
{
}

class Post
{
    /**
     * @use HasOwner<User>
     */
    use HasOwner;
}

/**
 * @template TItem
 */
class OwnedItem
{
    /**
     * @use HasOwner<TItem>
     */
    use HasOwner;
}

interface Serializable
{
}

/**
 * @require-implements Serializable
 */
trait CanBeSerialized
{
}

class Message implements Serializable
{
    use CanBeSerialized;
}

class Model
{
}

/**
 * @require-extends Model
 */
trait HasDatabaseId
{
}

class User extends Model
{
    use HasDatabaseId;
}

/**
 * @inheritors PermittedUser
 */
trait RestrictedTrait
{
}

class PermittedUser
{
    use RestrictedTrait;
}
