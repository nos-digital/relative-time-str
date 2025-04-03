# nos-rust-toolchain
Een repo met standaard files die je nodig zal hebben om een rust repository op te zetten.
Het idee van deze setup is afgekeken van (Jon Gjengset)[https://youtu.be/xUH-4y92jPg?si=_0E3FhZSr4rnnLwr].

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
