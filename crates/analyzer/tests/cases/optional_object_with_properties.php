<?php

class User
{
    public string $id = '';
    public string $name = '';
}

class CreateUserRequest
{
    public string $name = '';
}

class Author
{
    public string $id = '';
    public string $name = '';
    public string $email = '';
    public null|string $bio = null;
}

/**
 * @param object{name: string, id?: string} $user
 */
function log_user_action(string $action, object $user): void
{
    $id = $user->id ?? '<no id>';

    echo "$action performed by user {$user->name} with id $id\n";
}

/**
 * @param object{name: string, id?: string} $user
 */
function log_user_action2(string $action, object $user): void
{
    // @mago-expect analysis:non-existent-property - email will never exist! we know!
    $_email = $user->email;

    $id = $user->id; // @mago-expect analysis:possibly-non-existent-property
    if ($id === null) {
        $id = '<no id>';
    }

    echo "$action performed by user {$user->name} with id $id\n";
}

/**
 * @param object{name: string, id?: string, ...} $user
 */
function log_user_action3(string $action, object $user): void
{
    // @mago-expect analysis:ambiguous-object-property-access - email could exist, we don't know!
    $_email = $user->email;

    // @mago-expect analysis:possibly-invalid-argument - because our user is unsealed
    log_user_action($action, $user);
}

log_user_action('create', (object) ['name' => 'Alice']); // OK
log_user_action('update', (object) ['name' => 'Bob', 'id' => '123']); // OK
log_user_action('delete', new User()); // OK
log_user_action('create', new CreateUserRequest()); // OK
log_user_action('update', new Author()); // @mago-expect analysis:invalid-argument - extra properties

log_user_action2('create', (object) ['name' => 'Alice']); // OK
log_user_action2('update', (object) ['name' => 'Bob', 'id' => '123']); // OK
log_user_action2('delete', new User()); // OK
log_user_action2('create', new CreateUserRequest()); // OK
log_user_action2('update', new Author()); // @mago-expect analysis:invalid-argument - extra properties

log_user_action3('create', (object) ['name' => 'Alice']); // OK
log_user_action3('update', (object) ['name' => 'Bob', 'id' => '123']); // OK
log_user_action3('delete', new User()); // OK
log_user_action3('create', new CreateUserRequest()); // OK
log_user_action3('update', new Author()); // OK
