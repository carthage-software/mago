+++
title = "Contribuer"
description = "Comment configurer une copie locale de Mago, exécuter les tests et soumettre une modification."
nav_order = 30
nav_section = "Référence"
+++
# Contribuer à Mago

Merci d'envisager une contribution. Les étapes ci-dessous vous mènent d'une copie propre à une pull request.

## Pour commencer

1. Ouvrez une issue ou commentez une issue existante avant de vous lancer dans un travail conséquent. C'est le moyen le plus simple de vous assurer que votre travail correspond à la direction du projet.

2. Forkez le dépôt sur GitHub et clonez votre fork :

   ```bash
   git clone https://github.com/<your-username>/mago.git
   ```

3. Installez [Rust](https://www.rust-lang.org/tools/install) et [Just](https://github.com/casey/just), puis exécutez `just build` pour configurer le projet. Les utilisateurs de Nix peuvent lancer `nix develop` d'abord, puis `just build`.

4. Créez une branche :

   ```bash
   git checkout -b feature/my-awesome-change
   ```

5. Faites vos modifications en suivant le style de codage du projet.

6. Lancez les tests et le linter :

   ```bash
   just test
   just check
   ```

7. Committez et poussez :

   ```bash
   git commit -m "feat: add my awesome change"
   git push origin feature/my-awesome-change
   ```

8. Ouvrez une pull request contre le [dépôt principal](https://github.com/carthage-software/mago).

## Pull requests

Les corrections de bugs doivent inclure un test qui reproduit le bug. Les nouvelles fonctionnalités doivent inclure une couverture complète. En contribuant, vous acceptez que vos contributions soient sous la double licence MIT / Apache-2.0 du projet.

Pour signaler un problème de sécurité, suivez les étapes de la [politique de sécurité](https://github.com/carthage-software/mago/security/policy).
