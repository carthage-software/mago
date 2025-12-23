<?php

// ...

final class MenuFactory implements MenuFactoryInterface
{
    // ...

    private function generateMenuItemUrl(MenuItemDto $menuItemDto): string
    {
        // ...

        if (MenuItemDto::TYPE_CRUD === $menuItemType) {
            // ...

            $this->adminUrlGenerator
                // remove all existing query params to avoid keeping search queries, filters and pagination
                ->unsetAll()
                // set any other parameters defined by the menu item
                ->setAll($routeParameters);

            $mainTemplate = $this
                ->cleanupTemplate($template)
                // Cleanup head, we'll insert it after having parsed the DOM
                ->replaceRegex('/<head>((.|\n)*?)<\/head>/', '<head></head>');
        }
    }
}
