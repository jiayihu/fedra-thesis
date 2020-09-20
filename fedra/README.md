# Fedra

## Scenario

- Degli amici sono in spiaggia a divertirsi. Per immortalare il momento decidono di realizzare una video (story) tutti insieme. Una volta registrato il video, l'applicazione chiede all'utente di aggiungere i tag delle altre persone o di confermare gli utenti suggeriti tramite analisi del video. Confermati i partecipanti del video, l'utente può decidere i filtri da applicare al video.
- La computazione dei filtri avviene in maniera distribuita tra i partecipanti del video, che divengono implicitamente volunteer nella computazione. Il dispositivo utente da cui avviene l'editing è il master della computazione, che definisce sia la coordinazione che partecipa nella computazione nei limiti delle sue risorse. La distribuzione della computazione è un compito che richiede trasmissione sia di dati (il video) che di funzioni in quanto i filtri sono scaricabili e realizzabili dagli utenti di Instagram a disposizione per tutti tramite l'applicazione.
- Tuttavia qualora il dispositivo mobile non sia dotato di chip dedicato per le operazioni AI o abbiano batteria insufficiente, il software può richiedere la collaborazione di altri nodi nella rete dotati delle opportune risorse hardware (ad esempio GPU) tenendo il conto del constraint della latenza.
- Ogni dispositivo della rete espone un endpoint REST che descrive se stesso, fornendo informazioni quali potenza di calcolo disponibile e caratteristiche hardware come GPU, decoding efficiente di video/immagini, supporto istruzioni SIMD
- A seconda del filtro da applicare, il nodo master decide quale nodo volunteer è il più adatto, includendo se stesso tra le opzioni.
- All'improvviso i diversi ragazzi ricevono avvisi sui propri cellulari riguardo ad un imminente pioggia in arrivo grazie ai sensori di meteo di cui sono dotati le loro case. Si dirigono allora verso le proprie case.
- Nel tragitto uno di loro continua l'editing del video in macchina. I dispositivi mobile dei ragazzi rimangono partecipanti nella computazione, mentre altri nodi possono essere rimossi o aggiunti dinamicamente durante il movimento in base alla distanza fisica o alla latenza stimata
- Arrivati a casa di uno dei ragazzi, decidono di passare al desktop per avere uno schermo più grande con cui proseguire l'editing. A quel punto il desktop diventa master della computazione, sostituendo il dispositivo mobile che diviene semplice volunteer aggiuntivo e poiché sono collegati alla rete di casa, perfino il dispositivo IoT usato come sensore meteo può eseguire della computazione extra tramite istruzioni SIMD o usando eventuale GPU integrata se presente.
- **Elementi innovativi**:
    - Computazione distribuita a partecipazione dinamica (i nodi volunteer non sono decisi a priori e non conoscono le funzioni prima di unirsi alla computazione) e governata dall'utente
    - Cambio di master. Il master è un ruolo effimero, che può cambiare da un nodo all'altro sia per cause volontarie (trasferimento su desktop) che involontarie (crash del dispositivo). Chiaramente il supporto ai casi involontari complica il sistema in maniera significativa, è solo a scopo di esempio.
    - Trasferimento di dati e funzioni. Le funzioni sono sia scaricabili dalla rete (pull) che trasmissibili da un nodo all'altro (push)

## Tecnologie necessarie

Per abilitare questo scenario di computing nel continuum, ovvero un sistema in cui risorse e servizi distribuiti possono essere aggregati programmaticamente, sono necessarie alcune tecnologie abilitanti.

1. Permettere di eseguire programmi in un sistema di dispositivi eterogenei. Questo si divide nella necessità di avere:
    - Runtime per nodi browser per la parte del continuum esposta verso l'utente, in grado di definire il workflow. Tuttavia un nodo browser partecipa solo come volunteer deve possede il runtime anche per assolvere a tal compito.
    - Runtime per nodi cloud e fog, che eseguono in cluster di server. Idealmente supporta sia container che funzioni: i container permettono di fornire servizi, le funzioni di partecipare alla computazione distribuita.
    - Runtime leggeri per dispositivi embedded in grado di offrire concorrentemente sia un servizio che partecipazione alla computazione via funzioni. Il servizio è ad esempio l'analisi dei dati meteo, una funzione è un filtro Instagram.
2. Orchestratore che decide quali dispositivi o servizi comporre e ne esegue il deployment e scheduling. Nello scenario vengono solo composti dispositivi, non vi sono servizi esistenti utilizzati dinamicamente nel workflow. Ma è possibile che così come un dispositivo descriva le sue capacità hardware, lo faccia a maggior ragione anche per i servizi che già offre, siano questi in cloud (e.g. SaaS) o edge (e.g. misurazione temperatura ambiente).

## Implementazione proposta

La tesi implementerà una versione semplificata del sistema descritto sopra.

- Il runtime per i nodi browser è implementato all'interno di un browser attuale, Google Chrome e/o Firefox. Idealmente in futuro le applicazioni utente (mobile, tablet e desktop) sono tutte applicazioni web ed il sistema operativo è direttamente il browser (Ariadne), senza quindi incorrere nell'onere attuale ove le applicazioni web girano in un browser come processo in user-space dell'OS
- Un formato che permetta di trasmettere dati e funzioni, come rappresentazione di una risorsa REST. Le funzioni sono simili alle funzioni lambda del serverless-computing
- La computazione esegue diversi modelli leggeri di Deep Learning per immagini come esempio di filtri su video.
    - [CoCo SSD per l'object detection](https://github.com/tensorflow/tfjs-models/tree/master/coco-ssd)
    - [Blazeface per face recognition](https://github.com/tensorflow/tfjs-models/tree/master/blazeface)
    - [Body-pix per person segmentation](https://github.com/tensorflow/tfjs-models/tree/master/body-pix)
- La computazione esegue solo inferenza, non training. Al momento [Tensorflow supporta solo sperimentalmente WASM come back-end](https://blog.tensorflow.org/2020/03/introducing-webassembly-backend-for-tensorflow-js.html) e solo per alcune funzioni kernel (nel significato attribuito in ML, non nel mondo del system programming). Ciò significa che potenzialmente:
    - I modelli sopra potrebbero cambiare se utilizzano funzioni kernel non supportate dal back-end WASM
    - La libreria utilizzata potrebbe non essere Tensorflow in quanto rimane il problema del trasformare in WASM il codice front-end che definisce il modello. Un esempio di alternativa è [ONNX](https://github.com/microsoft/onnxjs) o [SSVM](https://github.com/second-state/ssvm).
    - I modelli potrebbero essere scritti from scratch in Rust nel caso in cui le strade precedenti non diano i risultati sperati. Chiaramente il valore del risultato sarebbe meno significativo rispetto al dimostrare di poter usare una libreria come Tensorflow, ma magari è possibile fare comunque un risultato discreto se fossero utilizzate istruzioni SIMD in Rust, che tuttavia è al momento solo una [RFC con implementazione sperimentale](https://github.com/rust-lang/rfcs/pull/2366).
    - Limitandosi all'inferenza, non vi è bisogno al momento di implementare uno stato globale condiviso che permetta un'efficiente training dei modelli.
    - Ogni funzione è eseguita da un solo nodo designato. Non vi è al momento provisioning automatico di ulteriori nodi nel caso di funzioni CPU-intensive.
- Il runtime per embedded, denominato Fedra, permette al dispositivo di partecipare alla computazione distribuita.
    - Fornisce endpoint REST che descrive le proprie risorse oltre alla possibilità, senza via REST, di ricevere dati e funzioni da eseguire.
    - Permette di eseguire funzioni in concorrenza. Il modello di concorrenza più opportuno, via thread o async ad esempio, è ancora da definire.
    - Possiede uno scheduler pensato appositamente per il sistema in oggetto. Anche in questo caso i dettagli sono ancora da definire, ad esempio se debba prediligere la reattività del servizio a discapito del response-time delle funzioni o se invece debba prediligere una esecuzione fair delle funzioni
    - Supporta STM32F4219 quindi con 2MB di memoria flash e 256KB di RAM. Dimostrare che il runtime può eseguire sulla famiglia STM32F4 è importante per la sua utilità per evitare che sia usabile solo con Raspberry Pi, le cui risorse hardware sono più vicine a quelle di un dispositivo utente che embedded. Anche a questo fine potrebbe essere necessario scrivere modelli più piccoli in Rust no-std invece che Tensorflow. STM32F429 è dotato però di Cortex-M4 che dovrebbe supportare istruzioni SIMD.
- Il runtime per cloud/fog è un porting di Fedra per x86_64. Ulteriori dettagli non sono conosciuti al momento, si spera solo in porting non doloroso ma gli imprevisti sono dietro l'angolo. Sicuramente il supporto ad architetture CPU diverse è considerato fin dall'inizio dello sviluppo di Fedra però.
    - Non supporta al momento container. Però nel caso di proseguimento dei lavori dopo la tesi, una prima aggiunta interessante sarebbe proprio l'implementazione dello standard WASI per chiamate di sistema e a seguire un layer di compatibilità per chiamate di sistema Linux in modo da supportare container. [BottleRocket di AWS](https://github.com/bottlerocket-os/bottlerocket) è una buona ispirazione, come pure [gVisor di Google](https://github.com/google/gvisor).
- Le diverse versioni di Fedra per embedded e cloud, oltre che potenzialmente essere la base per Ariadne ricordano le edizioni di Fedora, ovvero Fedora Workstation, Fedora CoreOS, Fedora IoT. Inoltre "Fedora" e "Fedra" distano come parole solo della vocale "o", motivo che ha portato al nome Fedra nella versione italiana invece che Phaedra. Le conoscenze di Fedora però sono limitate alla pagina Wikipedia al momento.
- Implementazioni esistenti:
    - [FAASM](https://www.usenix.org/conference/atc20/presentation/shillaker)
        - Orientato per HPC con stato globale condiviso e strategie per condividere anche lo stato locale. Tuttavia è orientato per il serverless, ovvero provisioning delle risorse per una sola funzione, mentre lo scenario della tesi presenta diverse funzioni appartenenti un workflow.
        - Esegue come processo Linux, di cui sfrutta i meccanismi di sicurezza come namespace e iptables
    - [Wascc Lattice](https://wascc.dev/docs/lattice/howitworks/)

        ![images/fig4.png](images/fig4.png)

        - L'immagine sopra è un lattice globale, dove ogni funzione è un attore e le capabilities sono processi separati con cui gli attori comunicano via messaggio. I messaggi sono inviati via protocollo NATS.

        ![images/assembly-mechs--beyond-wasmdome-(1).png](images/assembly-mechs--beyond-wasmdome-(1).png)

        - L'immagine sopra descrive una rete lattice dove i nodi sono orchestrati da Kubernetes tramite la libreria [Krustlet](https://github.com/deislabs/krustlet) installata su ogni nodo. I nodi però comunicano tra di loro via [NATS](https://nats.io/) senza Kubernetes. La rete rappresenta un gioco alla [crobots](https://en.wikipedia.org/wiki/Crobots) con player distribuiti, ognuno avente una strategia diversa.
        - Il lattice di Wascc è l'implementazione più vicina al sistema dello scenario descritto. Possiede infatti nodi distribuiti nella rete, che eseguono codice portabile WASM. Wascc inoltre supporta anche hot swapping dei moduli attori senza downtime.
        - I suoi difetti sono tuttavia possibilmente i seguenti:
            - Non è pensato per il sistema dello scenario, ad esempio non sono presenti nodi browser. Quest'ultimi si limitano a mostrare l'interfaccia del gioco (WebUI in immagine), senza essere parte fondamentale del sistema.
            - È una libreria e quindi richiede OS sottostante per poter operare. Inoltre significa che ottimizzazioni come uno scheduler pensato appositamente per il sistema devono essere implementate in user-space.
            - È orchestrato da Kubernetes che si occupa di fare provisioning, non di decidere, comporre e governare i nodi necessari per un workflow. Tuttavia in realtà il lattice Wascc non assume utilizzo necessariamente con Kubernetes.
            - Comunica via messaggi tramite protocollo NATS. Questa forma di comunicazione non è ricca come REST, che permette di definire in maniera uniforme risorse e servizi assieme ad una interfaccia comune per operare su questi. Ciò è fondamentale per un sistema eterogeneo come quello della tesi. Inoltre non è verificata la bontà del protocollo NATS per dispositivi embedded, rispetto ad esempio [CoAP](https://ieeexplore.ieee.org/document/6159216) che implementa REST su UDP.
            - Non si può nascondere che comprendere quali siano i reali difetti di Wascc rispetto al sistema dello scenario sia importante. Attualmente sembrerebbe che lo svantaggio principale sia essere una libreria, che quindi non può eseguire bare-metal ed esser compilata senza libreria standard. Questo lo rende quindi non idoneo per l'embedded con dispositivi resource-constrained, però a parte ciò sembra un'implementazione esistente molto interessante da cui prendere spunto.
