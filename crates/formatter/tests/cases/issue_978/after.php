<?php declare(strict_types=1);

class A
{
    private function a()
    {
        $veryVeryVeryVeryLongVariableName = $this->entityManager->getRepository(VeryVeryVeryLongEntityName::class)->findOneBy([], [
            'id' => 'desc',
        ]);
    }
}
