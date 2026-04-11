<?php

final class Named
{
    public string $nom = '';
}

final class PartenaireModel
{
    /** @return list<Named> */
    public function findAllByPk(array|int|null $ids): array
    {
        return [];
    }
}

final class Partenaire
{
    public static function model(): PartenaireModel
    {
        return new PartenaireModel();
    }
}

final class EditeurModel
{
    /** @return list<Named> */
    public function findAllByPk(array|int|null $ids): array
    {
        return [];
    }
}

final class Editeur
{
    public static function model(): EditeurModel
    {
        return new EditeurModel();
    }
}

final class CollectionModel
{
    public function findByPk(int $id): ?Named
    {
        return null;
    }
}

final class Collection
{
    public static function model(): CollectionModel
    {
        return new CollectionModel();
    }
}

final class Command
{
    public function setFetchMode(int $mode): self
    {
        return $this;
    }

    public function queryScalar(array $params = []): string
    {
        return '';
    }

    /** @return list<string> */
    public function queryColumn(): array
    {
        return [];
    }

    /** @return array<int, string> */
    public function queryAll(): array
    {
        return [];
    }
}

final class Db
{
    public function createCommand(string $sql): Command
    {
        return new Command();
    }
}

final class App
{
    public Db $db;

    public function __construct()
    {
        $this->db = new Db();
    }
}

final class Yii
{
    public static function app(): App
    {
        return new App();
    }
}

final class CHtml
{
    public static function encode(string $text): string
    {
        return $text;
    }

    public static function link(string $text, array $url, array $htmlOptions = []): string
    {
        return $text;
    }
}

final class Titre
{
    /** @var array<int, string> */
    public const array ENUM_SCIENTIFIQUE = [0 => 'non', 1 => 'oui'];
}

final class Category
{
    public string $categorie = '';
    public int $id = 0;
    public int $profondeur = 0;
}

final class SearchTitre
{
    public const int MONIT_YES = 1;
    public const int MONIT_NO = 2;
    public const int MONIT_IGNORE = 3;

    public int $scientifique = 0;
    public string $titre = '';
    public string $langues = '';
    public bool $languesEt = false;
    public string $issn = '';
    public string $hdateModif = '';
    public string $hdateVerif = '';
    public bool $vivant = false;
    public int $politique = 0;
    public bool $abonnement = false;
    public bool $accesLibre = false;
    public bool $owned = false;
    public string $aboCombine = '';
    public bool $sansAcces = false;
    public bool $monitoredByMe = false;
    public int $monitored = 0;
    public ?int $collectionId = null;
    public string $groupe = '';
    public ?int $issnpaysid = null;
    /** @var list<int> */
    public array $paysId = [];
    /** @var list<int> */
    public array $editeurId = [];
    /** @var list<int> */
    public array $aediteurId = [];
    /** @var list<int> */
    public array $ressourceId = [];
    /** @var list<int> */
    public array $suivi = [];
    /** @var list<int> */
    public array $detenu = [];
    public bool $categorie = false;
    /** @var list<int> */
    public array $cNonRec = [];
    public bool $categoriesEt = false;
    public bool $sanscategorie = false;
    /** @var list<int> */
    public array $acces = [];
    /** @var list<int> */
    public array $lien = [];
    /** @var list<int> */
    public array $attribut = [];
    public int $grappe = 0;

    /** @return list<Category> */
    public function getCategories(): array
    {
        return [];
    }
}

function code_to_full_name(string $code): string
{
    return $code;
}

final class Summary
{
    /**
     * @return array<array-key, mixed>
     */
    public function asArray(SearchTitre $st): array
    {
        $instName = 'example';
        $sum = [];
        if (isset(Titre::ENUM_SCIENTIFIQUE[$st->scientifique])) {
            $sum['scientifique'] = Titre::ENUM_SCIENTIFIQUE[$st->scientifique];
        }
        if ($st->titre) {
            $sum['Titre'] = $st->titre;
        }
        if ($st->langues) {
            if ($st->langues === 'aucune' || $st->langues === '!') {
                $sum['Langues'] = 'aucune';
            } else {
                $langCodes = preg_split('/[, ]\s*/', trim($st->langues, ', ')) ?: [];
                $langues = array_filter(array_map(code_to_full_name(...), $langCodes));
                $sum['Langues'] = join($st->languesEt ? ' ET ' : ' OU ', $langues);
            }
        }
        if ($st->issn) {
            $sum['ISSN'] = $st->issn;
        }
        if ($st->hdateModif) {
            $sum['Modifie'] = $st->hdateModif;
        }
        if ($st->hdateVerif) {
            $sum['Verifie'] = $st->hdateVerif;
        }
        if ($st->vivant) {
            $sum[] = 'Revue vivante';
        }
        if ($st->politique) {
            if ($st->politique > 0) {
                $sum[] = 'Avec une politique de publication';
            } elseif ($st->politique < 0) {
                $sum[] = 'Sans politique de publication';
            }
        }
        if ($st->abonnement && $st->accesLibre && $st->owned) {
            $sum[] = "Acces libre ou abonne ou disponible a {$instName}";
        } elseif ($st->abonnement && $st->accesLibre) {
            $sum[] = 'Acces libre ou abonne';
        } else {
            if ($st->accesLibre) {
                $sum[] = 'Acces libre';
            }
            if ($st->abonnement && $st->owned) {
                $sum[] = "abonne en ligne {$st->aboCombine} disponible a {$instName}";
            } else {
                if ($st->abonnement) {
                    $sum[] = ucfirst($instName) . ' est abonne en ligne';
                }
                if ($st->owned) {
                    $sum[] = "Disponible a {$instName}";
                }
            }
        }
        if ($st->sansAcces) {
            $sum[] = 'Sans acces';
        }
        if ($st->monitoredByMe) {
            $sum[] = "Suivi par {$instName}";
        }
        if ($st->monitored === SearchTitre::MONIT_YES) {
            $sum[] = 'Suivi';
        } elseif ($st->monitored === SearchTitre::MONIT_NO) {
            $sum[] = 'Non suivi';
        }
        if ($st->collectionId !== null) {
            $sum['Collection'] = (string) Collection::model()->findByPk($st->collectionId)?->nom;
        }
        // @mago-expect analysis:invalid-operand
        if (str_contains($st->groupe, 'pays') && $st->issnpaysid !== null && $st->paysId) {
            $pays = Yii::app()->db->createCommand(
                    'SELECT group_concat(nom) FROM Pays WHERE id IN (' . join(',', $st->paysId) . ') ORDER BY nom',
                )->queryScalar();
            $paysIssn = Yii::app()->db->createCommand(
                    'SELECT nom FROM Pays WHERE id = :id',
                )->queryScalar([':id' => (int) abs($st->issnpaysid)]) ?: 'SANS';
            $sum[] = "Pays ({$pays}) OU pays de publication ({$paysIssn})";
        } else {
            if ($st->paysId) {
                $sum['Pays'] = Yii::app()->db->createCommand(
                        'SELECT group_concat(nom) FROM Pays WHERE id IN (' . join(',', $st->paysId) . ') ORDER BY nom',
                    )->queryScalar();
            }
            if ($st->issnpaysid !== null) {
                $name = Yii::app()->db->createCommand(
                        'SELECT nom FROM Pays WHERE id = :id',
                    )->queryScalar([':id' => (int) abs($st->issnpaysid)]);
                $sum['Pays de publication'] = ($st->issnpaysid > 0 ? '' : 'SANS ') . $name;
            }
        }
        $getName = static fn(object $ar): string => (string) ($ar->nom ?? '');
        if ($st->editeurId) {
            $ids = array_map(abs(...), $st->editeurId);
            $names = array_map($getName, Editeur::model()->findAllByPk($ids));
            $sum['Editeur'] = join(' ; ', array_map(
                static fn(string $name, int $id): string => $id > 0 ? $name : "SANS {$name}",
                $names,
                $st->editeurId,
            ));
        }
        if ($st->aediteurId) {
            $sum['Editeur actuel ou precedent'] = join(' ; ', array_map(
                $getName,
                Editeur::model()->findAllByPk($st->aediteurId),
            ));
        }
        if ($st->ressourceId) {
            $ids = join(',', array_map(static fn(int $x): int => abs($x), $st->ressourceId));
            $ressources = Yii::app()->db->createCommand(
                    "SELECT id, nom FROM Ressource WHERE id IN ({$ids})",
                )->setFetchMode(\PDO::FETCH_KEY_PAIR)->queryAll();
            $l = [];
            foreach ($st->ressourceId as $rid) {
                if ($rid > 0 && isset($ressources[$rid])) {
                    $l[] = $ressources[$rid];
                } elseif ($rid < 0 && isset($ressources[0 - $rid])) {
                    $l[] = 'SAUF ' . $ressources[0 - $rid];
                }
            }
            $sum['Ressource'] = join(' ; ', $l);
            unset($l);
        }
        $suivi = $st->monitoredByMe ? [1, 2, 3] : $st->suivi;
        // @mago-expect analysis:invalid-operand
        if ($suivi && $st->monitored === SearchTitre::MONIT_IGNORE) {
            $sum['Suivi par'] = join(' ; ', array_map($getName, Partenaire::model()->findAllByPk($suivi)));
        }
        // @mago-expect analysis:invalid-operand
        if ($st->detenu && !$st->owned) {
            $sum['Disponible a'] = join('; ', array_map($getName, Partenaire::model()->findAllByPk($st->detenu)));
        }
        if ($st->categorie) {
            $cNonRec = $st->cNonRec;
            $sum['Thematique'] = join(
                $st->categoriesEt ? ' ET ' : ' OU ',
                array_map(
                    static fn(Category $x): string => (
                        $x->categorie . (in_array($x->id, $cNonRec, true) || $x->profondeur > 2 ? '' : ' (recursif)')
                    ),
                    $st->getCategories(),
                ),
            );
        }
        if ($st->sanscategorie) {
            $sum[] = 'Sans thematique';
        }
        if ($st->acces) {
            $accesToText = static function (int $v): string {
                $conv = ['', '', 'Texte integral', 'Resume', 'Sommaire', 'Indexation'];
                // @mago-expect analysis:mismatched-array-index
                return $conv[$v] ?? '';
            };
            $sum['Acces'] = join(' OU ', array_map($accesToText, $st->acces));
        }
        if ($st->lien) {
            $lienRequired = [];
            $lienProhibited = [];
            foreach ($st->lien as $l) {
                if ($l < 0) {
                    $lienProhibited[] = -$l;
                } elseif ($l > 0) {
                    $lienRequired[] = $l;
                }
            }
            if ($lienRequired) {
                $sum[] =
                    'Avec lien '
                    . join(
                        ', ',
                        Yii::app()->db->createCommand(
                                'SELECT nom FROM Sourcelien WHERE id IN (' . join(',', $lienRequired) . ')',
                            )->queryColumn(),
                    );
            }
            if ($lienProhibited) {
                $sum[] =
                    'Sans lien '
                    . join(
                        ', ',
                        Yii::app()->db->createCommand(
                                'SELECT nom FROM Sourcelien WHERE id IN (' . join(',', $lienProhibited) . ')',
                            )->queryColumn(),
                    );
            }
        }
        if ($st->attribut) {
            $attrRequired = [];
            $attrProhibited = [];
            foreach ($st->attribut as $a) {
                if ($a < 0) {
                    $attrProhibited[] = -$a;
                } elseif ($a > 0) {
                    $attrRequired[] = $a;
                }
            }
            if ($attrRequired) {
                $sum[] =
                    'Avec attribut '
                    . join(
                        ', ',
                        Yii::app()->db->createCommand(
                                'SELECT nom FROM Sourceattribut WHERE id IN (' . join(',', $attrRequired) . ')',
                            )->queryColumn(),
                    );
            }
            if ($attrProhibited) {
                $sum[] =
                    'Sans attribut '
                    . join(
                        ', ',
                        Yii::app()->db->createCommand(
                                'SELECT nom FROM Sourceattribut WHERE id IN (' . join(',', $attrProhibited) . ')',
                            )->queryColumn(),
                    );
            }
        }
        if ($st->grappe) {
            $gname = Yii::app()->db->createCommand('SELECT nom FROM Grappe WHERE id = '
                . (int) abs($st->grappe))->queryScalar();
            $gid = $st->grappe > 0 ? $st->grappe : 0 - $st->grappe;
            $sum[$st->grappe > 0 ? 'Grappe ' : 'SAUF grappe '] =
                '<span>'
                . CHtml::encode($gname)
                . CHtml::link(
                    '<span class="micon-new-window"></span>',
                    ['/grappe/view', 'id' => $gid],
                    [
                        'style' => 'padding-left:1ex',
                        'target' => '_blank',
                        'title' => "Voir la grappe \"{$gname}\" dans une nouvelle page",
                    ],
                )
                . '</span>';
        }
        return $sum;
    }
}
