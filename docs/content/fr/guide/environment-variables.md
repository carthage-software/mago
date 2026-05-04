+++
title = "Variables d'environnement"
description = "Toutes les variables d'environnement que Mago lit, ce qu'elles font et leur place dans la chaîne de précédence."
nav_order = 50
nav_section = "Guide"
+++
# Variables d'environnement

Mago lit un petit ensemble de variables d'environnement. Certaines surchargent des clés de `mago.toml`, les autres contrôlent le runtime (logs, couleurs, recherche du fichier de configuration).

## Runtime

### `MAGO_LOG`

Niveau de log. Utile pour déboguer un résultat inattendu.

Valeurs : `trace`, `debug`, `info`, `warn`, `error`.

```sh
MAGO_LOG=trace mago lint
```

### `NO_COLOR`

Mettez à n'importe quelle valeur véridique pour désactiver toute sortie colorée. Suit la convention [no-color.org](https://no-color.org/).

```sh
NO_COLOR=1 mago lint
```

### `FORCE_COLOR`

Mettez à n'importe quelle valeur véridique pour forcer la sortie colorée même quand stdout n'est pas un terminal. A priorité sur `NO_COLOR`. Suit la convention [force-color.org](https://force-color.org/).

```sh
FORCE_COLOR=1 mago lint | less -R
```

### `XDG_CONFIG_HOME`

Mago suit la [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/) pour trouver une configuration globale lorsqu'aucun fichier au niveau projet n'existe. La chaîne de repli est :

1. `$XDG_CONFIG_HOME/mago.toml` (si défini).
2. `$HOME/.config/mago.toml`.
3. `$HOME/mago.toml`.

Définir `XDG_CONFIG_HOME` change le premier répertoire de recherche.

```sh
XDG_CONFIG_HOME=/path/to/config mago lint
```

## Le préfixe réservé `MAGO_`

Mago se réserve le préfixe `MAGO_`. Seules les variables documentées sur cette page sont officiellement reconnues. Tout le reste préfixé par `MAGO_` est réservé à un usage interne et peut être ignoré silencieusement ou réutilisé dans une future version.

> Les versions antérieures mappaient automatiquement chaque variable `MAGO_*` dans l'arbre de configuration, si bien que quelque chose comme `MAGO_LINT=1` plantait avec une erreur « unknown field ». Mago 1.25 a restreint cela à la liste explicite ci-dessous.

## Surcharges de configuration

Ces variables surchargent la clé correspondante dans `mago.toml`. Elles ne couvrent que les scalaires de premier niveau ; il n'y a pas de support env-var pour les paramètres imbriqués comme les niveaux de règle individuels. Utilisez le fichier de configuration (ou une couche `extends`) pour cela.

### `MAGO_PHP_VERSION`

Surcharge `php-version`. Utile pour tester le même code contre plusieurs versions PHP sans modifier la configuration.

```sh
MAGO_PHP_VERSION=8.2 mago lint
```

### `MAGO_THREADS`

Surcharge `threads`.

```sh
MAGO_THREADS=4 mago lint
```

### `MAGO_STACK_SIZE`

Surcharge `stack-size`, en octets. Les valeurs hors plage sont ramenées à la fenêtre prise en charge (minimum 2 MiB, maximum 8 MiB).

```sh
MAGO_STACK_SIZE=8388608 mago lint
```

### `MAGO_EDITOR_URL`

Surcharge `editor-url` et l'URL d'éditeur détectée automatiquement. Entrée de plus haute précédence pour les chemins de fichiers cliquables dans la sortie de diagnostic. Voir la [section intégration éditeur](/guide/configuration/#editor-integration) pour les modèles pris en charge.

```sh
MAGO_EDITOR_URL="phpstorm://open?file=%file%&line=%line%&column=%column%" mago lint
```

### `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION`

Surcharge `allow-unsupported-php-version`. Mettez à `true` pour laisser Mago s'exécuter sur une version PHP qu'il ne prend pas officiellement en charge. Non recommandé.

```sh
MAGO_ALLOW_UNSUPPORTED_PHP_VERSION=true mago lint
```

### `MAGO_NO_VERSION_CHECK`

Surcharge `no-version-check`. Mettez à `true` pour désactiver l'avertissement émis quand le binaire installé diverge de la version épinglée dans `mago.toml`. Une divergence de version majeure reste fatale quelle que soit cette variable : tout l'intérêt d'un épinglage majeur est de bloquer les exécutions à travers des schémas de configuration incompatibles.

```sh
MAGO_NO_VERSION_CHECK=true mago lint
```
