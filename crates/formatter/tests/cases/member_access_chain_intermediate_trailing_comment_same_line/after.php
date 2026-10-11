<?php

final class RetrieveMediaActionTest
{
    public function test(): void
    {
        $this->entityManager->expects($this->exactly(3)) // 1 initial + 2 for variants
            ->method('flush')
        ;
    }
}
