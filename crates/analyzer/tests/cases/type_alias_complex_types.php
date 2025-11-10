<?php

/**
 * @type UserId int
 * @type UserName string
 * @type UserRecord array{id: UserId, name: UserName, active: bool}
 * @type UserList array<int, UserRecord>
 * @type OptionalUser UserRecord|null
 */
class UserRepository
{
    /**
     * @param UserId $id
     * @return OptionalUser
     */
    public function findById(int $id): null|array
    {
        if ($id > 0) {
            return ['id' => $id, 'name' => 'Test', 'active' => true];
        }
        return null;
    }

    /**
     * @return UserList
     */
    public function getAll(): array
    {
        return [
            ['id' => 1, 'name' => 'Alice', 'active' => true],
            ['id' => 2, 'name' => 'Bob', 'active' => false],
        ];
    }
}

function use_int(int $value): void
{
    echo $value;
}

function use_string(string $value): void
{
    echo $value;
}

function use_bool(bool $value): void
{
    echo $value ? 'true' : 'false';
}

$repo = new UserRepository();
$user = $repo->findById(1);
if ($user !== null) {
    use_int($user['id']); // Should be int
    use_string($user['name']); // Should be string
    use_bool($user['active']); // Should be bool
}

$users = $repo->getAll();
foreach ($users as $u) {
    use_int($u['id']); // Should be int
    use_string($u['name']); // Should be string
    use_bool($u['active']); // Should be bool
}
