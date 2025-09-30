<?php

$user = $config->mapUser($this->factory, new ResourceOwner([
    'id' => 789,
    'email' => 'himmel@hero-party.brave',
    'name' => 'Himmel the Hero',
    'username' => 'himmel_hero',
    'avatar_url' => 'https://example.com/himmel.jpg',
]));
