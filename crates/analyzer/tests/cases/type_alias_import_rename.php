<?php

/**
 * @psalm-type UserArray array{name: string, id: int}
 */
class UserModel
{
}

/**
 * @psalm-import-type UserArray from UserModel as UserData
 */
class UserService
{
    /**
     * @param UserData $data
     */
    public function save(array $data): void
    {
        echo $data['name']; // Should be string
        echo $data['id']; // Should be int
    }

    /**
     * @return UserData
     */
    public function load(): array
    {
        return ['name' => 'David', 'id' => 123];
    }
}

$service = new UserService();
$service->save(['name' => 'Eve', 'id' => 456]);
$loaded = $service->load();
echo $loaded['name']; // Should be string
echo $loaded['id']; // Should be int
