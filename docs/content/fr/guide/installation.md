+++
title = "Installation"
description = "Installer Mago via le script shell, un téléchargement manuel, Docker, ou le gestionnaire de paquets de votre langage."
nav_order = 20
nav_section = "Guide"
+++
# Installation

Mago est livré sous forme de binaire statique unique. Choisissez la méthode d'installation qui convient à votre environnement.

## Script shell (macOS, Linux)

La voie recommandée sur macOS et Linux. Le script détecte votre plateforme, récupère l'archive de release correspondante et dépose le binaire dans votre PATH.

Avec `curl` :

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash
```

Avec `wget` :

```sh
wget -qO- https://carthage.software/mago.sh | bash
```

### Épingler une version spécifique

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --version=1.25.2
```

La même syntaxe fonctionne avec `wget`.

### Vérifier le téléchargement

Si la [GitHub CLI](https://cli.github.com/) est dans votre PATH, le script vérifie l'archive contre l'attestation de build GitHub de Mago avant de la décompresser. Aucun drapeau requis. Si `gh` est absent ou trop ancien, le script affiche un avis et continue sans vérification.

Pour rendre la vérification obligatoire, passez `--always-verify`. Le script s'interrompt avant de toucher votre PATH si `gh` est indisponible, trop ancien ou si l'attestation ne correspond pas.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://carthage.software/mago.sh | bash -s -- --always-verify
```

Pour désactiver entièrement, passez `--no-verify`. Les deux drapeaux sont mutuellement exclusifs.

## Téléchargement manuel

La voie recommandée sur Windows et un bon repli sur tout système sans `bash`.

1. Ouvrez la [page des releases](https://github.com/carthage-software/mago/releases).
2. Téléchargez l'archive correspondant à votre système d'exploitation. Le nom suit `mago-<version>-<target>.tar.gz` (ou `.zip` sous Windows).
3. Extrayez l'archive et placez le binaire quelque part dans votre PATH.

Si vous gardez l'archive, vous pouvez la vérifier vous-même avant l'extraction.

```sh
VERSION=1.25.2
TARGET=x86_64-unknown-linux-gnu  # adjust for your platform
ASSET=mago-${VERSION}-${TARGET}.tar.gz

gh release download "$VERSION" --repo carthage-software/mago --pattern "$ASSET"
gh attestation verify "$ASSET" \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml

tar -xzf "$ASSET"
sudo mv "mago-${VERSION}-${TARGET}/mago" /usr/local/bin/
```

Une vérification réussie affiche `Verification succeeded!` et l'exécution de workflow qui a produit l'archive.

L'attestation est liée à l'archive, pas au binaire extrait. Si vous n'avez gardé que le binaire, vous ne pouvez pas le vérifier directement. Re-téléchargez l'archive, vérifiez-la, et comparez le `sha256sum` du binaire interne à celui déjà sur votre système.

## Docker

L'image officielle est construite à partir de `scratch` et pèse environ 26 Mo. Elle s'exécute partout où Docker s'exécute, prend en charge `linux/amd64` et `linux/arm64`, et ne nécessite aucun runtime PHP hôte.

```sh
docker run --rm -v $(pwd):/app -w /app ghcr.io/carthage-software/mago lint
```

Les tags incluent `latest`, des versions exactes et des épinglages progressivement plus larges (par exemple `1.25.2`, `1.25`, `1`). La [recette Docker](/recipes/docker/) couvre les exemples CI et les limitations à connaître.

## Gestionnaires de paquets

Ces voies sont pratiques mais dépendent de calendriers de publication externes qui sont souvent en retard sur la release GitHub. Après une installation par l'une d'elles, lancez [`mago self-update`](/guide/upgrading/) pour récupérer le dernier binaire officiel.

### Composer

Pour les projets PHP :

```sh
composer require --dev "carthage-software/mago:^1.25.2"
```

Le paquet Composer est un fin wrapper. Le premier appel à `vendor/bin/mago` télécharge le binaire pré-construit correspondant depuis la release GitHub et le met en cache. Les appels suivants réutilisent le cache et ne font aucune requête réseau.

Si la limite de taux anonyme de GitHub bloque le premier téléchargement (fréquent sur les runners CI partagés), définissez `GITHUB_TOKEN` ou `GH_TOKEN` pour cette invocation. Dans GitHub Actions, le jeton n'est pas exporté automatiquement, donc passez-le explicitement :

```yaml
- run: vendor/bin/mago lint
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Homebrew

La formule maintenue par la communauté est souvent en retard sur la release officielle. Installez-la, puis lancez `mago self-update` immédiatement.

```sh
brew install mago
mago self-update
```

### Cargo

La publication sur Crates.io peut avoir quelques heures de retard sur une release. Même schéma qu'avec Homebrew.

```sh
cargo install mago
mago self-update
```

## Vérification des releases en détail

Chaque archive de release (tarball par plateforme, tarball source, zip source et bundle WASM) est signée au moment du build via [`actions/attest-build-provenance`](https://github.com/actions/attest-build-provenance). La signature est une attestation [in-toto](https://in-toto.io/) stockée sur GitHub et liée à l'exécution de workflow qui a produit l'artefact, donc un téléchargement vérifié est prouvablement octet-identique à ce qui est sorti du pipeline de release de Mago.

Le script shell choisit l'un de trois modes selon les drapeaux que vous passez.

| Mode | Drapeau | Comportement |
| :--- | :--- | :--- |
| `auto` | aucun | Vérifie si `gh` est disponible ; sinon installe sans vérifier. |
| `always` | `--always-verify` | La vérification est obligatoire. Un `gh` manquant ou trop ancien, ou une correspondance échouée, interrompt l'installation. |
| `never` | `--no-verify` | Saute la vérification même quand `gh` est disponible. |

Quand la vérification s'exécute, le script lance :

```sh
gh attestation verify <archive> \
  --repo carthage-software/mago \
  --signer-workflow carthage-software/mago/.github/workflows/cd.yml
```

L'épinglage `--signer-workflow` est important. Il lie l'attestation au fichier exact du workflow de release. Un jeton GitHub Actions compromis qui pourrait déclencher un workflow différent dans le même dépôt échouerait quand même à la vérification.

Si la vérification échoue, le script copie l'archive non vérifiée dans votre répertoire de travail courant sous `<file>.unverified.tar.gz` (afin qu'elle survive au nettoyage du répertoire temp et que vous puissiez l'inspecter), affiche une erreur en rouge et quitte avant l'extraction. Rien n'atteint votre PATH.

L'appel de vérification lit depuis l'API publique des attestations, donc aucun `gh auth` n'est requis. Vous avez seulement besoin d'un `gh` récent qui inclut la sous-commande `gh attestation`.

### Épingler le script d'installation

`https://carthage.software/mago.sh` redirige vers [`scripts/install.sh`](https://github.com/carthage-software/mago/blob/main/scripts/install.sh) sur la branche `main`. Les futures révisions sont prises automatiquement, ce qui est pratique mais signifie aussi que les futurs changements du script arrivent sans préavis.

Pour une hygiène de chaîne d'approvisionnement plus stricte, épinglez le script à un commit spécifique que vous avez relu :

```sh
COMMIT=cd4cf4dfdbc72bd028ad26d11bcc815a49e27e9a  # replace with a commit you have read
curl --proto '=https' --tlsv1.2 -sSf \
  "https://raw.githubusercontent.com/carthage-software/mago/${COMMIT}/scripts/install.sh" \
  | bash -s -- --always-verify
```

GitHub ne réécrit pas les fichiers à un SHA donné, donc les octets que vous avez relus sont les octets que vous exécutez. Mettre l'épinglage à jour est un acte délibéré : relisez le nouveau commit, puis bumpez `COMMIT`.

## Vérifier l'installation

```sh
mago --version
```

Si cela affiche une version, vous êtes prêt à [exécuter Mago contre votre code](/guide/getting-started/).
