<?php

declare(strict_types=1);

enum PersonEntityType: string
{
    case INDIVIDUAL = 'individual';
    case CORPORATION = 'corporation';
}

class PersonEntity
{
    private PersonEntityType $type = PersonEntityType::INDIVIDUAL;
    private null|string $companyName = null;
    private null|string $firstName = null;
    private null|string $lastName = null;

    public function setType(PersonEntityType $type): void
    {
        $this->type = $type;
    }

    public function setCompanyName(string $name): void
    {
        $this->companyName = $name;
    }

    public function setFirstName(string $firstName): void
    {
        $this->firstName = $firstName;
    }

    public function setLastName(string $lastName): void
    {
        $this->lastName = $lastName;
    }
}

function createPersonEntity(
    null|string $companyName = null,
    null|string $firstName = null,
    null|string $lastName = null,
): PersonEntity {
    $entity = new PersonEntity();

    if ($companyName !== null) {
        $entity->setType(PersonEntityType::CORPORATION);
        $entity->setCompanyName($companyName);
    } elseif ($firstName !== null && $lastName !== null) {
        $entity->setType(PersonEntityType::INDIVIDUAL);
        $entity->setFirstName($firstName);
        $entity->setLastName($lastName);
    }

    return $entity;
}

function test(): void
{
    $corp = createPersonEntity(companyName: 'Acme Inc');

    $person = createPersonEntity(
        firstName: 'John',
        lastName: 'Doe',
    );

    $default = createPersonEntity();
}
