<?php

function canView($user, $contact): bool
{
    return (
        $user->can('has_service', [$user->agency, config('navigation.main_menu.developers.services')])
            && (
                $contact->developer?->id === auth()->user()->agency?->developer?->id
                && in_array($contact->id, $contact->developer?->attachedAgents->pluck('id')->all())
            )
    );
}

$canView =
    $user->can('has_service', [$user->agency, config('navigation.main_menu.developers.services')])
        && (
            $contact->developer?->id === auth()->user()->agency?->developer?->id
            && in_array($contact->id, $contact->developer?->attachedAgents->pluck('id')->all())
        );

function canEdit($user, $contact): bool
{
    return $user->isAdmin() && ($contact->owner?->id === $user->id && $contact->isEditable());
}
