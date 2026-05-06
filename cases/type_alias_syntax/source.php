<?php

namespace App {
    /**
     * @psalm-type      UserId          int
     * @phpstan-type    UserName    =   string
     * @type            UserEmail   =   string
     *
     * @type UserData array{
     *     id: UserId,
     *     name: UserName,
     *     email: UserEmail
     * }
     */
    class User
    {
        /**
         * @var UserData
         */
        public array $data;

        /**
         * @param UserId $id
         * @param UserName $name
         * @param UserEmail $email
         */
        public function __construct(int $id, string $name, string $email)
        {
            $this->data = [
                'id' => $id,
                'name' => $name,
                'email' => $email,
            ];
        }

        /**
         * @return UserData
         */
        public function getData(): array
        {
            return $this->data;
        }

        /**
         * @param UserData $data
         */
        public function setData(array $data): void
        {
            $this->data = $data;
        }
    }
}

namespace Other {
    use App;
    use App\User;
    use App\User as RenamedUser;

    /**
     * @phpstan-import-type     UserId      from User
     * @psalm-import-type       UserName    from User
     * @import-type             UserEmail   from User
     */
    class UserService
    {
        /**
         * @param UserId $id
         * @param UserName $name
         * @param UserEmail $email
         */
        public function createUser($id, $name, $email): \App\User
        {
            return new \App\User($id, $name, $email);
        }
    }

    /**
     * @param !\App\User::UserId $id
     * @param !\App\User::UserName $name
     * @param !\App\User::UserEmail $email
     */
    function createUser1($id, $name, $email): \App\User
    {
        return new \App\User($id, $name, $email);
    }

    /**
     * @param !App\User::UserId $id
     * @param !App\User::UserName $name
     * @param !App\User::UserEmail $email
     */
    function createUser2($id, $name, $email): \App\User
    {
        return new \App\User($id, $name, $email);
    }

    /**
     * @param !User::UserId $id
     * @param !User::UserName $name
     * @param !User::UserEmail $email
     */
    function createUser3($id, $name, $email): \App\User
    {
        return new \App\User($id, $name, $email);
    }

    /**
     * @param !RenamedUser::UserId $id
     * @param !RenamedUser::UserName $name
     * @param !RenamedUser::UserEmail $email
     */
    function createUser4($id, $name, $email): \App\User
    {
        return new \App\User($id, $name, $email);
    }
}
