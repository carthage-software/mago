<?php

return match ($this) {
    self::January,
    self::March,
    self::May,
    self::July,
    self::August,
    self::October,
    self::December,
        => 31,
    self::February => 28,
};

return match ($this) {
    self::January,
    self::March,
    self::May,
    self::July,
    self::August,
    self::October,
    self::December,
        => 31,
    self::February => 28,
    self::April, self::June, self::September, self::November => 30,
};

class A
{
    public function __construct(
        private int $a,
    ) {
    }
}

class User
{
    private string $firstname;
    private string $lastname;

    public function __construct(
        // Fullname should be concatenated
        private string $fullname {
            get() {
                // Return the full name
                return $this->firstname . ' ' . $this->lastname;
            }
            set($value) {
                $parts = explode(' ', $value); // Split by space

                $this->firstname = $parts[0]; // First part is firstname
                $this->lastname = $parts[1]; // Second part is lastname
            }
        },

        // Email should be validated
        private string $email,

        // Password should be hashed
        private string $password,
        // more fields
    ) {
    }
}
