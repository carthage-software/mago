<?php

class User
{
    public function getId(): int
    {
        return 1;
    }

    public function getName(): string
    {
        return 'some_user';
    }
}

class ReportGroup
{
    /** @return list<User> */
    public function getUsers(): array
    {
        return [new User()];
    }

    public function getPercentage(): int
    {
        return 50;
    }
}

class Report
{
    /** @return list<ReportGroup> */
    public function getGroups(): array
    {
        return [new ReportGroup(), new ReportGroup()];
    }
}

class Checker
{
    public function check(Report $report): void
    {
        $percentagesByUser = [];

        foreach ($report->getGroups() as $group) {
            foreach ($group->getUsers() as $user) {
                if (!isset($percentagesByUser[$user->getId()])) {
                    $percentagesByUser[$user->getId()] = [
                        'user' => $user,
                        'total' => 0,
                    ];
                }

                $percentagesByUser[$user->getId()]['total'] += $group->getPercentage();
            }
        }

        foreach ($percentagesByUser as $data) {
            if (100 !== $data['total']) {
                echo
                    sprintf(
                        'Total percentage for user \'%s\' must be 100, but got %d',
                        $data['user']->getName(),
                        $data['total'],
                    )
                ;
            }
        }
    }
}
