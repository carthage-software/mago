+++
title = "Mettre à jour"
description = "Garder Mago à jour avec la commande self-update, y compris épingler une version spécifique ou se synchroniser avec le champ version de mago.toml."
nav_order = 80
nav_section = "Guide"
+++
# Mettre à jour

`mago self-update` remplace le binaire en cours d'exécution par une release plus récente. À utiliser pour les installations issues du script shell, de Homebrew, de Cargo ou d'un téléchargement manuel.

> Les installations Composer sont différentes. Le wrapper Composer épingle un binaire qui correspond à la version du paquet Composer, donc vous mettez Mago à jour avec `composer update` plutôt qu'avec `self-update`.

## Flux courants

Vérifier les mises à jour sans installer :

```sh
mago self-update --check
```

La commande affiche la nouvelle version (s'il y en a une) et quitte avec un code non nul lorsqu'une mise à jour est disponible, ce qui la rend scriptable en CI.

Mettre à jour vers la dernière release :

```sh
mago self-update                  # interactive confirmation
mago self-update --no-confirm     # skip the prompt
```

Épingler une version spécifique :

```sh
mago self-update --tag 1.25.1
```

## Synchroniser avec l'épinglage de version du projet

Si votre `mago.toml` utilise l'[épinglage de version](/guide/configuration/#version-pinning), vous pouvez synchroniser le binaire installé avec ce que le projet attend sans saisir la version vous-même :

```sh
mago self-update --to-project-version
```

Pour un épinglage exact (`version = "1.25.1"`), cela résout directement vers ce tag de release. Pour un épinglage majeur ou mineur, Mago parcourt les releases GitHub récentes et installe la plus haute qui satisfait toujours l'épinglage. Ainsi, `version = "1"` avec 2.0 déjà sortie installe quand même la dernière release 1.x. `version = "1.14"` avec du 1.19.x dans la nature redescend vers le dernier 1.14.x.

La commande échoue uniquement si aucune release publiée ne satisfait l'épinglage.

## Référence

```sh
Usage: mago self-update [OPTIONS]
```

| Drapeau | Description |
| :--- | :--- |
| `--check`, `-c` | Vérifie les mises à jour sans installer. Quitte avec un code non nul lorsqu'une mise à jour est disponible. |
| `--no-confirm` | Saute la confirmation interactive. |
| `--tag <VERSION>` | Installe un tag de release spécifique au lieu du plus récent. Mutuellement exclusif avec `--to-project-version`. |
| `--to-project-version` | Installe ce que l'épinglage `version` du projet exige. Échoue si aucun épinglage n'est défini. Mutuellement exclusif avec `--tag`. |
| `-h`, `--help` | Affiche l'aide et quitte. |

Les drapeaux globaux doivent venir avant `self-update`. Voir l'[aperçu CLI](/fundamentals/command-line-interface/) pour la liste complète.
