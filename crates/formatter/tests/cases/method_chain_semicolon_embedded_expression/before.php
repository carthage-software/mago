<?php

class C
{
    public function hasGames(array $team): bool
    {
        return $this->getEntityManager()->getRepository(Game::class)
            ->createQueryBuilder('game')
            ->select('COUNT(game.id)')
            ->getQuery()
            ->getSingleScalarResult() > 0
        ;
    }

    public function slug(string $value): string
    {
        return (string) new AsciiSlugger()->slug($value ?? '')
            ->lower()
        ;
    }
}
