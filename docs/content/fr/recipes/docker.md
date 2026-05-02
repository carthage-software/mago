+++
title = "Recette Docker"
description = "Exécuter Mago dans n'importe quel environnement sans l'installer localement."
nav_order = 60
nav_section = "Recettes"
+++
# Recette Docker

L'image conteneur officielle est construite à partir de `scratch` avec un binaire lié statiquement, donc l'image est petite (environ 26 Mo) et n'embarque pas d'OS.

## Image

L'image se trouve à `ghcr.io/carthage-software/mago` sur le GitHub Container Registry.

## Tags

Chaque release publie plusieurs tags pour épingler à la précision souhaitée :

| Tag | Exemple | Description |
| :--- | :--- | :--- |
| `latest` | `ghcr.io/carthage-software/mago:latest` | Pointe toujours vers la release la plus récente. |
| `<version>` | `ghcr.io/carthage-software/mago:1.25.1` | Épinglé à une version exacte. |
| `<major>.<minor>` | `ghcr.io/carthage-software/mago:1.25` | Suit le dernier patch d'une version mineure. |
| `<major>` | `ghcr.io/carthage-software/mago:1` | Suit la dernière release d'une version majeure. |

L'image prend en charge `linux/amd64` et `linux/arm64`. Docker récupère la bonne variante pour votre hôte.

## Démarrage rapide

Montez votre répertoire de projet et lancez n'importe quelle commande :

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

## Exemples

Lint :

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

Vérifier le formatage sans écrire :

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt --check
```

Appliquer le formatage :

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago fmt
```

Lancer une analyse statique :

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago analyze
```

Afficher la version :

```sh
docker run --rm ghcr.io/carthage-software/mago --version
```

## Intégration CI

### GitHub Actions

```yaml
name: Mago Code Quality

on:
  push:
  pull_request:

jobs:
  mago:
    name: Run Mago Checks
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/carthage-software/mago:1
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Check formatting
        run: mago fmt --check

      - name: Lint
        run: mago lint --reporting-format=github

      - name: Analyze
        run: mago analyze --reporting-format=github
```

L'image n'inclut pas PHP ni Composer. Cela fonctionne très bien pour le formateur et le linter. L'analyseur a besoin que les dépendances Composer du projet soient installées pour résoudre correctement les symboles ; sans cela, il signalera des faux positifs sur les symboles indéfinis. Si votre projet dépend de paquets tiers et que vous voulez lancer l'analyseur, préférez une [installation native](/guide/installation/) avec les dépendances Composer installées.

### GitLab CI

```yaml
mago:
  image: ghcr.io/carthage-software/mago:1
  script:
    - mago fmt --check
    - mago lint
    - mago analyze
```

### Bitbucket Pipelines

```yaml
pipelines:
  default:
    - step:
        name: Mago Code Quality
        image: ghcr.io/carthage-software/mago:1
        script:
          - mago fmt --check
          - mago lint
          - mago analyze
```

## Alias shell

Traiter l'image comme s'il s'agissait d'un binaire local :

```sh
alias mago='docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago:1'
```

Ajoutez la ligne à votre fichier d'init de shell, rechargez le shell, puis lancez `mago lint` (ou toute autre sous-commande) comme d'habitude.
