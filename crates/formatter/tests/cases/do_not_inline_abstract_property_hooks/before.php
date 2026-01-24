<?php

interface BookUrlInfo
{
    public int $id { get; }
    public ?string $slug { get; set; }
}
