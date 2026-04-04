<?php

class Abc
{
    public function test(): void
    {
        $x = $a ?? [];

        $view->emailNotifications = $this->stringUtils->splitStringToArray($jobPosting->getVacancyEmailNotification())
            ?? [];

        $result =
            $this->someService->checkSomethingReallyLongMethodName($firstArg, $secondArg)
            || $this->fallbackCheck($otherArg);

        return $this->stringUtils->splitStringToArray($jobPosting->getVacancyEmailNotification()) ?? [];

        $this->value = $this->getFromCache($key) ?? $this->getFromDatabase($reallyLongParameterName) ?? [];
    }
}
