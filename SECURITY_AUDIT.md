# Rapport d'Audit de Sécurité & Guide de Durcissement — UniLauncher

**Version auditée :** `v1.1.0-beta.1`  
**Branche Git :** `beta`  
**Date :** 15 novembre 2026  
**Auteur :** Antigravity Security & Systems Engineering  

---

## 1. Résumé Exécutif & Contexte de l'Alerte

Lors du déploiement de l'installateur d'UniLauncher, Microsoft Defender a émis l'alerte suivante :
- **Menace identifiée :** `Trojan:Win32/Bearfoos.A!ml`
- **ThreatID :** `2147731250`
- **SeverityID :** `5` (Critique)
- **Fichier ciblé :** `UniLauncherSetup.exe`
- **Statut :** Quarantaine automatique

### Comprendre le suffixe `!ml` (Machine Learning)
Le suffixe `!ml` indique explicitement que la détection provient d'un **modèle heuristique / apprentissage automatique** de Windows Defender Cloud & Client, et **non d'une signature statique connue** d'un code malveillant spécifique.  
Lorsqu'un exécutable PE (Portable Executable) inconnu présente une combinaison de comportements ou d'attributs suspects (absence de signature numérique, pas de manifeste Windows, création de raccourcis via PowerShell masqué, modification de clés de registre, exécution de commandes console en arrière-plan), le classifieur attribue un score de risque élevé et classe préventivement le binaire comme cheval de Troie générique (`Bearfoos.A!ml` ou `Wacatac.B!ml`).

Notre audit approfondi confirme qu'UniLauncher **ne contient aucun code malveillant ni backdoor**. Cependant, le binaire et son installeur utilisaient plusieurs pratiques considérées à haut risque par les moteurs antivirus modernes.

---

## 2. Analyse des Déclencheurs Heuristiques & Failles Identifiées

| # | Vecteur / Pratique identifiée | Mécanisme de détection Antivirus / Risque | Niveau de risque |
|---|---|---|---|
| **1** | **Absence de Manifeste d'Application PE (`app.manifest`)** | Les exécutables contenant les chaînes `setup` ou `install` dans leur nom déclenchent l'heuristique de détection d'installeur Windows UAC. Sans manifeste déclarant explicitement `asInvoker`, Windows tente une virtualisation de compatibilité et le classe comme exécutable non standard. | **Élevé** |
| **2** | **Absence de Métadonnées `VERSIONINFO`** | L'exécutable compilé ne contenait pas de bloc de ressources Windows officiel (`CompanyName`, `FileDescription`, `LegalCopyright`, `ProductVersion`, `OriginalFilename`). Les droppers malveillants générés à la volée ont généralement une table de ressources vide. | **Élevé** |
| **3** | **Création de Raccourcis via PowerShell masqué** | `setup.rs` exécutait : `powershell.exe -NoProfile -NonInteractive -Command "$ws = New-Object -ComObject WScript.Shell; ..."` avec le flag `CREATE_NO_WINDOW (0x08000000)`. C'est un schéma classique de dropper / vecteur d'attaque surveillé par AMSI (Antimalware Scan Interface). | **Critique** |
| **4** | **Exécution de `cmd.exe /c start` pour lancer des URLs/jeux** | `scan.rs` utilisait l'interpréteur de commandes Windows pour ouvrir des URLs ou lancer des désinstallations, créant un risque d'injection de commandes et un signalement de process injection. | **Élevé** |
| **5** | **Invocation de `powercfg.exe` en arrière-plan** | Le mode d'optimisation gaming exécutait `powercfg.exe /setactive <GUID>` en créant un sous-processus console caché. Les chevaux de Troie utilisent souvent des outils système natifs (`living-off-the-land binaries` / LOLBins). | **Moyen** |
| **6** | **Arrêt de processus via `taskkill.exe`** | L'installeur et le désinstalleur invoquaient `taskkill.exe /F /IM UniLauncher.exe` et `tasklist.exe` au lieu d'utiliser les APIs Win32 directes. | **Moyen** |
| **7** | **Mises à jour sans vérification d'intégrité cryptographique** | L'auto-updater téléchargeait un exécutable dans `%TEMP%` et l'exécutait sans calculer ni comparer son empreinte SHA-256 avec une source de confiance. | **Critique** |

---

## 3. Plan de Durcissement & Mesures Implémentées

### 3.1 Intégration du Manifeste Officiel (`app.manifest`)
Un fichier de manifeste conforme aux spécifications Microsoft a été intégré directement dans la section de ressources du PE :
- Déclare le niveau d'exécution minimal sans élévation de privilèges inutile :
  ```xml
  <requestedExecutionLevel level="asInvoker" uiAccess="false" />
  ```
- Déclare explicitement la compatibilité avec Windows 10 et Windows 11 (GUIDs OS).
- Active le support DPI par moniteur v2 (`PerMonitorV2`) et la compatibilité chemins longs (`longPathAware`).

### 3.2 Intégration des Métadonnées PE (`build.rs` & `winres`)
Un script de build Rust (`build.rs`) injecte automatiquement le bloc `VERSIONINFO` lors de la compilation :
- `ProductName` : `UniLauncher (BETA)`
- `FileDescription` : `UniLauncher - Lanceur universel de jeux PC (BETA)`
- `CompanyName` : `nowxyii3`
- `LegalCopyright` : `Copyright (C) 2026 nowxyii3. Licence MIT.`
- `ProductVersion` & `FileVersion` : `1.1.0-beta.1`
- `OriginalFilename` : `UniLauncher.exe`

### 3.3 Élimination Complète des Invocations PowerShell pour les Raccourcis
- Suppression intégrale de l'appel à `powershell.exe`.
- Utilisation de la bibliothèque native Rust `mslnk` pour forger directement la structure binaire des fichiers `.lnk` (Shell Link Binary File Format - MS-SHLLINK).
- **Résultat :** 0 sous-processus spawned, 0 interaction avec PowerShell, 0 déclenchement d'AMSI.

### 3.4 Sécurisation de l'Exécution d'URIs & Remplacement de `cmd.exe`
- Remplacement des appels `cmd.exe /c start` par la fonction native `crate::process::open_uri()`.
- Implémentation d'une liste blanche stricte des protocoles autorisés (`steam:`, `com.epicgames.launcher:`, `uplay:`, `origin2:`, `ea:`, `battlenet:`, `xbox:`, `ms-windows-store:`, `ms-settings:`, `https:`, `http:`).
- Rejet immédiat de tout schéma non autorisé (`file:`, `javascript:`, `cmd:`, `powershell:`, etc.).

### 3.5 Gestion Directe de l'Énergie via l'API Win32 (PowrProf.dll)
- Remplacement des sous-processus `powercfg.exe` par l'appel direct à l'API système C :
  ```rust
  PowerSetActiveScheme(std::ptr::null_mut(), &scheme_guid);
  ```
- **Résultat :** bascule instantanée sans aucun processus enfant suspect.

### 3.6 Gestion Interne des Processus via `sysinfo`
- Remplacement des commandes `taskkill` et `tasklist` par l'inspection et la terminaison in-process via la crate `sysinfo::System`.

### 3.7 Durcissement Cryptographique de l'Auto-Updater
- **Calcul SHA-256 à la volée :** chaque fichier téléchargé est lu et haché avec `sha2::Sha256`.
- **Validation stricte contre `checksums.txt` :** si l'empreinte ne correspond pas exactement au manifeste de release, le fichier est immédiatement supprimé et le processus interrompu.
- **Whitelist d'URLs :** seuls les serveurs officiels (`github.com/nowxyii3/Unilancher` et `objects.githubusercontent.com`) sont autorisés pour le téléchargement.
- **Prise en charge du canal Beta :** sélection du canal (`Stable` ou `Beta`) avec parsing SemVer adapté aux suffixes `-beta.X`.

---

## 4. Isolation Complète de l'Environnement BETA

Pour garantir que vos données réelles et vos paramètres stables ne soient **jamais écrasés ou altérés** lors des tests :

| Composant | Environnement Stable | Environnement BETA |
|---|---|---|
| **Dossier de configuration** | `%APPDATA%\UniLauncher` | `%APPDATA%\UniLauncher-Beta` |
| **Dossier d'installation** | `%LOCALAPPDATA%\Programs\UniLauncher` | `%LOCALAPPDATA%\Programs\UniLauncher-Beta` |
| **Clé de désinstallation Registre** | `HKCU\...\Uninstall\UniLauncher` | `HKCU\...\Uninstall\UniLauncher-Beta` |
| **Nom du raccourci Bureau** | `UniLauncher.lnk` | `UniLauncher (BETA).lnk` |
| **AppId Inno Setup** | `{5E2F424E-8622-4806-DF08-6364400A202F}` | `{7C4A1E92-9D33-4D2A-B981-54E38F929C44}` |
| **Nom de l'exécutable d'installation** | `UniLauncherSetup.exe` | `UniLauncher-Beta-Setup.exe` |

*Note de sécurité :* Lors du tout premier lancement en version Beta, si aucune configuration beta n'existe, l'application peut lire les paramètres stables en **lecture seule** comme base de départ sans jamais les modifier.

---

## 5. Guide de Signature Numérique (Authenticode)

Bien que nos corrections éliminent les déclencheurs comportementaux suspects, **tout binaire Windows non signé distribué sur Internet subit un malus de réputation SmartScreen et Defender**. La signature numérique de code (Authenticode) est le moyen le plus efficace de garantir l'authenticité et d'éliminer définitivement les faux positifs.

### 5.1 Pourquoi la signature élimine le faux positif
- Elle associe cryptographiquement l'exécutable à une identité de développeur vérifiée.
- Elle garantit que le fichier n'a pas été altéré ou infecté après sa compilation.
- Elle permet d'établir une réputation globale dans l'écosystème Windows SmartScreen.

### 5.2 Solutions de Certificat Recommandées

1. **Azure Trusted Signing (Recommandé - Moderne & Économique) :**
   - Service cloud managé de Microsoft (anciennement Microsoft Azure Code Signing).
   - Coût très réduit (~10$ / mois d'utilisation) sans avoir besoin d'acheter un token matériel USB (dongle FIPS).
   - Intégration native dans GitHub Actions via l'action officielle `azure/trusted-signing-action`.

2. **Certificat Commercial OV / EV (Certum, Sectigo, DigiCert) :**
   - Certificat de signature de code standard (OV) ou à validation étendue (EV).
   - Réputation SmartScreen immédiate avec EV.
   - Nécessite la signature via `signtool.exe`.

### 5.3 Procédure de Signature avec `signtool.exe`
```powershell
# Commande standard pour signer UniLauncher
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /a "dist\UniLauncher-Beta-Setup.exe"

# Vérification de la signature
signtool verify /pa /v "dist\UniLauncher-Beta-Setup.exe"
```

### 5.4 Soumission de Faux Positif à Microsoft Security Intelligence
Si un faux positif persiste avant l'obtention d'un certificat commercial :
1. Rendez-vous sur le portail officiel de soumission Microsoft :  
   👉 **https://www.microsoft.com/en-us/wdsi/filesubmission**
2. Sélectionnez **"Software developer"**.
3. Déposez l'exécutable `UniLauncher-Beta-Setup.exe` avec son empreinte SHA-256.
4. Remplissez les informations du produit (UniLauncher, Open Source, GitHub : https://github.com/nowxyii3/Unilancher).
5. Dans les commentaires, mentionnez :  
   *"False positive detection Bearfoos.A!ml on an open-source gaming library manager written in Rust. Clean compilation, manifest and metadata included, 0 malware behavior."*
6. Microsoft analyse généralement les soumissions de développeurs sous 24 à 48 heures et pousse une mise à jour des définitions cloud Defender pour retirer la détection.

---

## 6. Procédure de Test de la Version BETA

1. **Accéder à GitHub Actions sur le dépôt :**
   - Allez sur l'onglet **Actions** du dépôt GitHub : https://github.com/nowxyii3/Unilancher/actions
   - Sélectionnez le workflow **"Build & Test UniLauncher BETA"**.
2. **Télécharger l'artefact de test :**
   - Téléchargez l'artefact compressé nommé `UniLauncher-Beta-v1.1.0-beta.1-Windows`.
3. **Vérifier l'empreinte SHA-256 :**
   ```powershell
   Get-FileHash UniLauncher-Beta-Setup.exe -Algorithm SHA256
   ```
   Comparez avec la ligne correspondante dans `checksums.txt`.
4. **Exécuter l'installation BETA :**
   - Lancez `UniLauncher-Beta-Setup.exe`.
   - Observez le chemin cible par défaut (`%LOCALAPPDATA%\Programs\UniLauncher-Beta`).
   - Constatez que vos données stables dans `%APPDATA%\UniLauncher` restent intactes.
5. **Vérifier le comportement de Windows Defender :**
   - Windows Defender doit accepter le fichier sans le bloquer ni le mettre en quarantaine.
