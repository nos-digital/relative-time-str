# nos-rust-toolchain
Een repo met standaard files die je nodig zal hebben om een rust repository op te zetten.
Het idee van deze setup is afgekeken van [Jon Gjengset](https://youtu.be/xUH-4y92jPg?si=_0E3FhZSr4rnnLwr).

> [!IMPORTANT]
> Als je ervoor kiest om de bestanden via een PR op de default branch te zetten, dan moet je de PR niet met een squash of rebase mergen. Als je dit doet zal de history van de commits op deze repository verloren gaan en zal je in den einde der tijden meer merge conflicts hebben tijdens het updaten van de toolchain files.

# Hoe voeg ik dit toe aan mn repo of update mijn repo?
Om een nieuw rust project op te zetten moet je de volgende stappen uitvoeren in de git repo:
```shell
$ # Als je dit nog niet gedaan hebt
$ git remote add toolchain git@github.com:nos-digital/nos-rust-toolchain.git
$ git fetch toolchain
$ # Optioneel: Als je het als een losse PR wil mergen
$ git switch --create toolchain-setup
$ git merge --allow-unrelated toolchain/master
$ git push --set-upstream
```

Deze repo pint de Rust-versie niet met een `rust-toolchain.toml`; zet in plaats daarvan
`rust-version` in de `[package]` sectie van je `Cargo.toml`:

```toml
rust-version = "1.99.1" # hetzelfde als `rust-version` in .github/workflows/ci.yml
```

# GitHub Actions
Repositories bouwen en deployen met GitHub Actions. De `Dockerfile` is alleen de
runtime image (distroless glibc) waar CI de binaries in zet; het compileren gebeurt
in CI. Sectie 4 van `ARCHITECTURE.md` legt uit hoe dat werkt.

Een repository die nog op Tekton draait, verhuist zo:
1. Merge `toolchain/master` zoals altijd (zie hierboven). Dat brengt de nieuwe
   `Dockerfile` en `.github/` mee. Je oude Tekton-`Dockerfile` geeft daarbij een
   conflict: neem de versie uit de toolchain.
2. Zet in de `Dockerfile` je eigen binaries in plaats van `my_compiled_binary`.
   Voeg runtime-libraries toe als je die nodig hebt; sectie 4 van `ARCHITECTURE.md`
   legt uit hoe.
3. Vul in `.github/workflows/ci.yml` je binaries, image-naam en common-infra map in
   (`my_compiled_binary`, `my-app`). Die roept de herbruikbare
   [rust-ci workflow](https://github.com/nos-digital/nos-github-actions#rust-ciyml)
   van nos-github-actions aan. Alle opties (MariaDB, sqlx, ...) staan in de README
   van nos-github-actions; nos-image-repository is een ingevuld voorbeeld.

   De base image in de `Dockerfile` is op digest gepind;
   `.github/dependabot.yml` laat Dependabot daar wekelijks updates voor voorstellen.
   `.github/workflows/claude-review.yml` doet een korte review door Claude op elke
   nieuwe PR en heeft het `ANTHROPIC_API_KEY` secret nodig.
4. Verwijder in nos-common-infra de Tekton-pipelines van de repository (de ArgoCD
   Application in `tekton/app/argocd-apps/` en `tekton/app/backend/<repo>/`). Vervang een verplichte `Tekton` status check in
   de branch rules door de nieuwe checks (`ci / ...`) en verwijder de Tekton-webhook
   van de repository.

Let op: in GitHub Actions gelden de NOS-lints uit `.cargo/config.toml` echt (de
Tekton-image zette `RUSTFLAGS`, waardoor ze wegvielen), en clippy faalt op
warnings. Reken op lint-meldingen in de eerste PR.

Een nieuwe Rust-versie gaat in deze volgorde: eerst de images in nos-common-infra
(`.github/images.json`), dan deze toolchain, en dan per repository
`rust-version` in `Cargo.toml` en in `.github/workflows/ci.yml` in dezelfde PR.
