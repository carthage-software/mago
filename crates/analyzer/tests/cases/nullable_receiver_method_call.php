<?php

declare(strict_types=1);

final class Title {
    public string $val = 'x';

    public function getVal(): string {
        return $this->val;
    }
}

final class Item {
    public function getTitle(): Title {
        return new Title();
    }
}

final class OtherItem {
    public function getTitle(): Title {
        return new Title();
    }
}

function plainCall(?Item $item): Title {
    return $item->getTitle(); // @mago-expect analysis:possible-method-access-on-null
}

function chain(?Item $item): string {
    return $item->getTitle()->val; // @mago-expect analysis:possible-method-access-on-null
}

function chainMethod(?Item $item): string {
    return $item->getTitle()->getVal(); // @mago-expect analysis:possible-method-access-on-null
}

function takeTitle(Title $t): string {
    return $t->val;
}

function callAsArg(?Item $item): string {
    return takeTitle($item->getTitle()); // @mago-expect analysis:possible-method-access-on-null
}

function unionReceiver(Item|OtherItem|null $item): Title {
    return $item->getTitle(); // @mago-expect analysis:possible-method-access-on-null
}
