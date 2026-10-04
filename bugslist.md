## Buglist : Mise à jour à la fermeture, Toast de cooldown et Upload (Doublons corrigés)

### 2. Suppression des doublons de notifications in-app et du cooldown (45s)

- **Problème / Constat :** Les notifications in-app (« jeu start », « jeu fermé, save dans 45s », « upload automatique », « upload terminé ») ainsi que le toast de cooldown apparaissent systématiquement en double à l'écran, en raison d'appels multiples rapprochés, concurrents ou de composants montés en double (ex: double écouteur d'événements).
- **Comportement attendu :** Un unique toast doit pouvoir être émis pour chaque événement dans une fenêtre temporelle donnée afin d'éradiquer tout doublon et spam visuel dans le panneau de notifications.
- **Tâches techniques :**
  - Identifier la fonction centrale ou le gestionnaire global déclenchant l'émission des toasts (démarrage, fermeture/cooldown, upload).
  - Mettre en place un mécanisme de verrouillage global par horodatage (`timestamp`) ou un verrou `useRef` pour filtrer et bloquer l'émission de toasts identiques ou consécutifs sur un intervalle très court.
  - S'assurer que les écouteurs d'événements (IPC / WebSockets / State listeners) ne s'enregistrent pas en double.

---
