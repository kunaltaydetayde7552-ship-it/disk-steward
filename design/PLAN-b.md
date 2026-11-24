---
title: Plano de design - Disk Steward, direção B (Proveta)
status: proposta em arena
fonte-de-verdade: docs/API.md (contrato Overview / Target / Preview / RunResult / ScheduleAgent / DockerUsage)
referencia-de-acabamento: attack-shark-x3 (design/PLAN.md, design/screenshots)
prototipo: .design-arena/variantes/claude-b.html
tokens: design/tokens-b.css
---

# Plano de design — Disk Steward, direção B: "Proveta"

> Preenchido antes da primeira linha do protótipo e revisado depois de cada rodada de screenshots
> (seção 8). Esta é só a direção B; a A está em `PLAN-a.md`. Depois da escolha do Lucas, o vencedor
> é fundido em `design/PLAN.md` e `design/tokens.css`.
>
> **B é só escura, de propósito.** A direção A ocupa a superfície clara; B existe para mostrar a
> outra metade do espaço de decisão. Um tema claro de B não foi desenhado e não deve ser inferido.

## 0. Projeto

- **Stack e versões (alvo):** Tauri v2 + Vue 3 (`<script setup lang="ts">`) + Vite + Tailwind CSS v4 (CSS-first, sem `tailwind.config.js`) + `motion-v` + `vue-i18n` + Pinia. O único backend é `crates/steward-core` (Rust), que chama o CLI `auto-compress-apfs`. Nenhuma camada web existe ainda.
- **Design system / tokens existentes:** nenhum. O protótipo usa os nomes de campo de `docs/API.md` (`freeBytes`, `onDiskBytes`, `privateBytes`, `risk`, `riskDetail`, `freedBytes`, `lastExitStatus`, `reclaimableBytes`...), então a UI real troca o mock por `invoke()` sem renomear nada.
- **Componentes que já resolvem parte disto:** nenhum componente. Da referência attack-shark-x3 herda-se o **nível de acabamento e o esqueleto de janela** (sidebar com pílula elevada, linhas com ícone em círculo, dock persistente com uma ação primária). Não se herda paleta, fonte nem objeto.
- **Convenções a seguir:** código, tokens e identificadores em inglês; texto de interface em pt-BR com acento correto. Números com `Intl.NumberFormat('pt-BR')`: GiB com uma casa (`75,2`), MiB inteiro abaixo de 1 GiB (`727 MiB`), milhar com ponto (`23.703`). Unidades binárias, as mesmas do `df -h` e do motor. Horários no fuso `America/Sao_Paulo` (`measuredAt` `2026-11-24T18:18:00Z` vira "15:18"). Loading e erro aparecem onde o dado aparece; o dock é a superfície única da ação pendente.
- **Restrições da stack:** fonte self-hosted via `@fontsource-variable/sofia-sans` (OFL). Janela com `titleBarStyle: Overlay` (semáforos sobre a sidebar), padrão 1180x760, mínimo 960x640. O protótipo usa `@tailwindcss/browser@4.3.5`, `motion@14.14.0` (ESM do jsDelivr, com fallback para WAAPI se o import falhar) e a fonte pelo jsDelivr **só no protótipo**.

## 1. Assunto

- **O que é, concretamente:** o utilitário de barra de menus de um MacBook com SSD de 460,4 GiB e 75,2 GiB livres. Mede para onde vai o disco de desenvolvimento, mantém caches e toolchains comprimidos com a compressão transparente do APFS, **prova** o ganho e **trava** o que sairia pela culatra (clones do uv e do Bun, pastas `target` do Cargo).
- **Para quem:** o Lucas, dev sênior. Sabe o que é clone APFS, `du`, `df` e launchd. Quer números honestos e um freio quando o ganho aparente é falso.
- **O único trabalho desta tela:** ler o disco como se lê uma proveta: **nível** (o que ocupa), **menisco fantasma** (o que ocuparia sem compressão) e **o que está dividido** (clones). Três tamanhos por pasta, no mesmo objeto, na mesma escala.
- **Vocabulário do assunto:** proveta graduada de borossilicato, graduações esmaltadas, menisco, leitura na altura do olho, sedimento, decantação, braço lateral, rolha, mesa de luz, lâmpada de sódio, lupa de leitura. Do lado do disco: volume, bloco, clone (`clonefile`), compressão transparente (decmpfs), conferência byte a byte, quarentena, agente do launchd, código de saída, venv, store, toolchain.

## 2. Cor

Cinco matizes, todos tirados da bancada de um laboratório à noite: o vidro âmbar do frasco de reagente, o esmalte branco das graduações, o brilho de borda do borossilicato sobre a mesa de luz, a lâmpada de sódio do corredor e o cobre nu. Cor é **luz de estado**, não decoração: o mesmo `--light` tinge o halo do chão, a mesa de luz, os grãos densos e a hachura de ganho.

| Token | Hex | De onde vem | Papel |
| --- | --- | --- | --- |
| `--color-ground` | `#2B1F17` | Vidro âmbar de frasco de reagente visto contra a bancada escura. | Chão da janela. |
| `--color-ground-deep` | `#1C140F` | O mesmo âmbar na sombra. | Poços, campos de comando, sidebar. |
| `--color-raised` / `-hi` | `#3A2B20` / `#433226` | Âmbar mais espesso, onde a luz atravessa duas paredes. | Pílula ativa, dock, linha selecionada. |
| `--color-line` | `#574233` | Aresta do vidro. | Só borda e régua, nunca texto. |
| `--color-enamel` | `#F2EBE3` | O esmalte branco queimado das graduações. | Texto. |
| `--color-enamel-muted` / `-faint` | `#C4B3A3` / `#A8927F` | Esmalte gasto. | Texto secundário e legenda. |
| `--color-boro` | `#8FDCCF` | O brilho frio da borda do borossilicato sobre a mesa de luz. | Saúde, ganho, foco e seleção. |
| `--color-sodium` | `#F0B04A` | Lâmpada de sódio de baixa pressão. | Bloqueado e pendente, calmo, nunca alarme. |
| `--color-copper` | `#FF8A70` | Fio de cobre recém-cortado, salmão sob a luz. | Exclusivo para erro. |

Os semáforos da janela usam as cores reais do macOS (não são paleta).

**Estados de luz** (`data-light` na janela; `--light` é `@property <color>` e transiciona):
`boro` (normal, saudável), `sodium` (algo bloqueado ou agendamento desligado), `copper` (execução falhou), `enamel` (medindo; luz neutra porque ainda não se sabe).

Contraste medido com `contraste.mjs` (todos os pares usados para texto estão acima de 4.5:1):

- corpo `enamel` sobre `ground`: **13.55:1**; sobre `raised-hi`: **10.31:1**.
- secundário `enamel-muted` sobre `ground`: **7.87:1**; sobre `raised-hi`: **5.99:1**.
- legenda `enamel-faint` sobre `ground`: **5.40:1**; sobre `raised`: **4.58:1**. Sobre `raised-hi` dá **4.11:1**, então `faint` **nunca** aparece em superfície `raised-hi` (linha selecionada de Alvos e subtítulo do dock usam `muted`).
- `boro` sobre `ground`: **10.15:1**; `sodium`: **8.40:1**; `copper`: **6.96:1**.
- texto do botão primário `ground` sobre `enamel`: **13.55:1**; `ground-deep` sobre `boro`: **11.51:1**.

## 3. Tipo

- **Sofia Sans Variable (OFL, peso 1..1000).** Escolhida pelo traço fino: nos pesos 180..250 os numerais parecem as graduações esmaltadas de uma proveta, finas e altas, e continuam legíveis em 120 px. É uma humanista levemente estreita, sem o desenho neutro de Inter ou Geist, e o "1" com bandeira e o "7" aberto leem como instrumento. Uma família só, três papéis por peso.
- **Papéis:** numeral de herói (peso ~200, `tabular-nums`) uma vez por tela; títulos 600; corpo 400..500; legendas 500. Distinção por peso e tamanho, **nunca** por caixa alta. Caminhos (`~/.cargo/registry`) e comandos em `ui-monospace`, só onde o texto é literalmente um caminho ou comando.
- **Escala:** razão ~1.25 no texto: legenda 12, rótulo 13, corpo 15, lead `clamp(16..18)`, título `clamp(22..28)`. Numerais saltam para escala de display: `numeral-md` `clamp(28..36)`, `numeral-lg` `clamp(52..84)`, `numeral-xl` `clamp(64..136)`. Todos os tamanhos de display em `clamp()` no `@theme`.
- **Medida de linha:** explicações até 60ch.
- **Self-hosting no app:** `@fontsource-variable/sofia-sans` importado no CSS de entrada.

## 4. Composição

### Arquitetura de informação

Sidebar com quatro destinos, na ordem da pergunta que a pessoa faz ao abrir:

1. **Visão geral** — quanto está livre, quanto a compressão devolveu, Docker e agendamento. Abre aqui.
2. **Alvos** — as oito pastas do catálogo, ligar e desligar, três tamanhos por pasta, por que algumas estão travadas.
3. **Execuções** — prévia, confirmação, execução ao vivo, resultado, falha, nada a fazer, histórico do dia.
4. **Agendamento** — os dois agentes do launchd (compressão diária e limpeza do Docker).

Mais o **menu da barra** (o app vive lá): leitura curta, uma ação, abrir, medir de novo, sair.

Por que é intuitivo:
- É o padrão de preferências do macOS (sidebar + conteúdo), então não há sistema a aprender. Atalhos `Cmd+1..4` no app (`1..4` no protótipo), `Cmd+Enter` para a ação do dock, `Esc` fecha, setas percorrem os alvos.
- **Um objeto só** em todas as telas: a mesma proveta, na mesma escala de leitura. Na Visão geral ela é o disco inteiro; em Alvos é a pasta selecionada; em Execuções é o que está sendo comprimido; em Agendamento é o que espera o agente. A pessoa aprende a ler uma vez.
- **O dock** fica no rodapé de toda tela e carrega a única ação primária e, quando ela está desabilitada, o motivo ao lado ("Tudo o que está ligado já está comprimido.").
- A sidebar mostra estado sem pedir clique: "3 ligados" em Alvos, "61%" em Execuções durante a execução, ponto de sódio em Agendamento quando a compressão diária está desligada.

### Wireframes (1180x760)

**Visão geral** (`#overview`, `#overview/measuring`, `#overview/docker-off`)

```
+----------+--------------------------------------------------------------+
| o o o    |  Macintosh HD, medido às 15:18                               |
| [] Disk  |  75,2  GiB livres                      75,2 GiB _.  ,---.    |
|  Steward |  de 460,4 GiB.                         Livre      |  ( 3,5 ) 388,7
|          |  Sem a compressão, seriam 71,7.        19,4 GiB _.|  `---'  sem compressão
| (Visão)  |                                        Alvos     _|   lupa  385,2
|  Alvos 3 |  (o) Compressão     De 4,9 para 1,4 ... 3,5  46,9 |  usados hoje
|  Execuç. |  (o) Alvos de dev   3 ligados, 2 ...   19,4  Docker|  proveta
|  Agend. *|  (o) Docker         Imagens 21,7 ...   14,6        |  graduada
|          |  (o) Agendamento    Só a limpeza ...  Desligado 318,9 GiB      |
| Medido ↻ |                                        Sistema e apps |         |
| [HD 75,2]|  +--------------------------------------------------------+   |
|          |  | (*) A compressão diária está desligada  [Ver] [Prévia] |   |
+----------+--------------------------------------------------------------+
```
Leituras à esquerda da proveta (livre, alvos, Docker, sistema), lupa à direita no menisco: o vão entre o menisco fantasma (388,7) e o nível real (385,2) é o ganho, 3,5 GiB. Medindo: banda de varredura sobe pela proveta, luz `enamel`, valor de Alvos vira esqueleto "medindo". Docker parado: camada Docker some, a etiqueta vira "Sistema e Docker", a linha diz "Parado / sem números" e explica como abrir o Docker Desktop.

**Alvos** (`#targets/<id>`)

```
+----------+-------------------------------------------------------------+
| sidebar  |  Alvos                                                      |
|          |  Cada pasta tem três medidas: ...                           |
|          | [Toolchains do Rust   1,3 GiB (on)]---.   |  Toolchain ~/.rustup
|          |  Registro do Cargo    1,1 GiB (on)    |  |  Toolchains do Rust
|          |  Cache do npm         1,1 GiB (on)    |  |  1,3 GiB no disco
|          |  Ferramentas do mise  1,4 GiB (off)   |  |  Era 3,1. 1,8 devolvidos
|          |  Cache do uv   7,4 [cad] (off)        |  |  ---- Lógico       3,1
|          |  Store do Bun  4,2 [cad] (off)     fio`->|  ==== No disco     1,3
|          |  Store do pnpm 2,9      (off)          |  ---- Só desta     1,3
|          |  Pastas target  não medido [cad]   proveta  14.120 de 16.884
|          |                                      3,1 lógico / 1,3 no disco
|          |  +--------------------------------------------------------+ |
|          |  | (o) 3 alvos ligados, 3 bloqueados     [Pré-visualizar] | |
+----------+-------------------------------------------------------------+
```
Um fio de luz sai da linha selecionada e entra na proveta no nível "no disco". Passar o mouse numa linha acende a camada correspondente. Bloqueados (`allowed: false`) têm cadeado, status em sódio e a proveta com rolha. uv e Bun ganham **braço lateral com frascos** (os venvs e os projetos que dividem os blocos por clone), com a parte compartilhada hachurada. `rust-target` é vidro tracejado com grãos girando: "não medido, muda a cada build", e o painel sugere `cargo clean`.

**Execuções** (`#runs`, `/confirm`, `/running`, `/result`, `/failed`, `/empty`)

```
+----------+-------------------------------------------------------------+
| sidebar  |        proveta        |  Prévia de 15:20                    |
|          |   ( grãos caindo )    |  727 MiB a comprimir                |
|          |   - - 2,1 no início   |  2.146 arquivos em 3 pastas, 38 ... |
|          |   ___ 1,9 agora       |  [vial] Registro do Cargo  412 MiB  |
|          |                       |  [vial] Cache do npm       236 MiB  |
|          |      mesa de luz      |  [vial] Toolchains do Rust  79 MiB  |
|          |                       |  Hoje  11:18  19.880 arquivos  2,9  |
|          |  +------------------------------------------------------+   |
|          |  | (o) 727 MiB em 2.146 arquivos   [Atualizar] [Comprimir]|  |
+----------+-------------------------------------------------------------+
```
Confirmar abre uma folha presa ao topo da janela (não um modal centralizado), com o que vai acontecer e o que não muda. Rodando: grãos grossos caem e decantam em grãos finos, o menisco desce, "61 % concluído", anel por pasta, dock com anel de progresso e "Parar depois deste arquivo". Resultado: "Concluída às 15:19, em 48 s", 402 MiB liberados, fantasma tracejado onde o nível estava. Falha: luz `copper`, "188 MiB liberados antes de parar", linhas de stderr do motor em monospace e "Abrir log". Vazio: "Nada a comprimir", com a sugestão concreta de ligar o mise e o botão Comprimir desabilitado.

**Agendamento** (`#schedule`, `#schedule/installed`)

```
+----------+-------------------------------------------------------------+
| sidebar  |  Agendamento                                    proveta      |
|          |  (o) Compressão diária          Não instalado    727 MiB      |
|          |      [auto-compress-apfs install-launchd  Copiar] esperando |
|          |      Agente / Última saída / Log                1,4 GiB      |
|          |  (o) Limpeza do Docker          Carregado       já comprimidos
|          |      Agente / Última saída / Log  [Abrir log]               |
|          |  +--------------------------------------------------------+ |
|          |  | (*) Compressão diária desligada [Instalar compressão]  | |
+----------+-------------------------------------------------------------+
```

**Menu da barra** (`#tray`, `#tray/running`): mini proveta + "75,2 GiB livres", três linhas (Compressão, Agendamento, Docker), um botão ("Pré-visualizar compressão" ou "Ver execução"), e Abrir / Medir de novo / Sair. Durante a execução aparece a linha com anel e porcentagem, e Sair vira "Sair depois da execução".

### Quebra estrutural e alinhamento

- A proveta **troca de lado** por tela (direita na Visão geral, entre lista e painel em Alvos, esquerda em Execuções, direita em Agendamento) e **viaja** entre os lugares com FLIP, em vez de cada tela ter seu gráfico.
- As leituras (etiquetas, fio, lupa) são uma camada HTML por cima do conteúdo, alinhada às graduações reais do SVG, com linhas de chamada que terminam na parede do vidro.
- Numerais grandes alinham pela linha de base com a unidade; valores das linhas alinham à direita.
- Responsivo: abaixo de 1100 px a sidebar vira trilho de ícones de 72 px; abaixo de 760 px vira navegação no topo, a proveta sobe para cima do conteúdo, as linhas empilham e o dock fica `sticky`. Sem rolagem horizontal a 360 px (medido).

## 5. Assinatura

- **Mecanismo: "a leitura do menisco".** Uma proveta de borossilicato sobre uma mesa de luz. As graduações são o eixo de dados. O conteúdo é sedimento: grãos finos e densos na cor da luz são o que está comprimido; grãos grossos de esmalte são o que não está; hachura é o que é dividido por clone. Um **menisco fantasma** tracejado marca o tamanho lógico (ou o "antes"). A distância entre o fantasma e o menisco real **é** o ganho.
- **Manifestações:**
  - Visão geral: o disco inteiro em camadas, com lupa no menisco mostrando os 3,5 GiB devolvidos.
  - Alvos: a pasta selecionada, com fio de luz até a linha, braço lateral com frascos para clones, rolha com luz de sódio para travados e vidro tracejado com grãos girando para o que não é medido.
  - Execuções: grãos caindo e decantando, menisco descendo ao vivo.
  - Agendamento: o que espera o agente.
  - Sidebar, dispositivo e menu da barra: a mini proveta.
  - A luz do estado tinge o halo do chão (`--hx` segue a proveta), a mesa e os grãos.
- **Curva de easing de assinatura:** `--ease-settle: cubic-bezier(0.22, 0.9, 0.24, 1)`, com 900 ms. É a decantação: o grão cai rápido e assenta numa cauda longa, sem overshoot (sedimento não quica). Usada nos níveis, na viagem da proveta, no fio e nos toggles.
- **Microinterações implementadas:** viagem FLIP da proveta entre telas; níveis transicionando; hover de linha acende a camada; fio de luz desenhado da linha à proveta; lupa entrando; luz de estado transicionando por `@property`; toggle com pulso no frasco da linha; desfazer no toast; grãos caindo durante a execução; banda de varredura ao medir; respiração do ponto do dock enquanto trabalha; anel de progresso por pasta e no dock; folha de confirmação descendo do topo. Tudo desliga com `prefers-reduced-motion` (os estados finais aparecem sem animação).
- **Por que isto não existe em outro produto:** utilitários de disco desenham sunburst (DaisyDisk), treemap (GrandPerspective), barra segmentada (Armazenamento do macOS) ou um orbe que pulsa (CleanMyMac). Nenhum desenha **três tamanhos no mesmo recipiente**, e nenhum mostra o que é dividido por clone como um vaso comunicante. Aqui o objeto responde exatamente à pergunta que o app existe para responder: "isso ocupa mesmo, ou só parece?".

## 6. Revisão contra os clichês

1. **Creme quente + serifa + terracota:** não. Escuro, sem serifa; o vermelho-salmão é só erro.
2. **Quase preto + acento ácido/neon:** evitado. O chão é âmbar saturado (`#2B1F17`), não preto; o acento é aqua de vidro iluminado, não neon, e só aparece como luz de estado.
3. **Jornal, fios finos, zero radius:** não. Raios por hierarquia (controle 10, pílula 12, linha 16, dock 20), sombras tingidas de âmbar, nunca `rgba(0,0,0,.1)`.
4. **Kit de cards de SaaS:** evitado. Nenhuma grade de cards; cada tela é um objeto e linhas de leitura.
5. **Cromo de template:** sem eyebrow em caixa alta, sem "A · B", sem seta em botão, sem roxo, sem monospace decorativo.

**"Em que produto já vi este layout, esta paleta ou esta assinatura?"**

- **Layout (sidebar + conteúdo + dock flutuante):** Ajustes do Sistema do macOS e a referência attack-shark-x3. Mantido de propósito, porque é um utilitário nativo e o esqueleto foi pedido como referência de acabamento. O que muda é o centro: em vez de formulário, um objeto que viaja.
- **Paleta:** a resposta honesta é **Gruvbox dark** (fundo marrom-acinzentado, aqua, amarelo e laranja), que tem a mesma família de matizes. A diferença é real (o chão aqui é âmbar com croma alto, não cinza quente; o aqua puxa para ciano de vidro, não para verde-oliva; a cor vem como luz emitida pela mesa e não como preenchimento de sintaxe), mas a família é parecida. Fica registrado como risco na seção 7.
- **Assinatura:** não encontrei produto de disco ou de sistema que use uma proveta graduada, menisco fantasma ou vaso comunicante para clones. A lupa de leitura lembra callouts de ilustração técnica (páginas de produto da Apple), mas aqui ela mostra um dado (o vão de 3,5 GiB), não um detalhe de hardware.
- **Toggle e semáforos:** padrão macOS, de propósito.

**O que mudou depois desta revisão:** os protótipos anteriores (barras e anéis) foram descartados porque respondiam "quanto ocupa", não "ocupa de verdade?"; a lupa foi redesenhada para mostrar só o vão e as duas leituras; o vermelho de erro deixou de ter origem inventada (o teste de chama do cobre é verde, não salmão) e passou a ser o cobre nu.

## 7. Riscos

- **A aposta mais ousada:** trocar o gráfico de cada tela por **um único objeto físico** que viaja entre telas e muda de leitura, num app que poderia ser uma tabela.
- **O que a torna defensável:** o objeto carrega informação que uma tabela não mostra junta (lógico, no disco, privado e compartilhado na mesma escala) e o texto continua lá: toda leitura da proveta também está numa linha ou etiqueta com número. A proveta é `aria-hidden`; o conteúdo acessível são as linhas.
- **Paleta parecida com Gruvbox:** se o Lucas ler assim, o primeiro ajuste é empurrar o chão para um âmbar mais vermelho (frasco de reagente mais escuro) e o aqua para o azul do borossilicato, mantendo os papéis.
- **Custo de pintura:** máscaras de grão (`feTurbulence`) num SVG grande. No app, gerar as texturas uma vez como imagem e animar só `transform`.
- **O que eu corto primeiro se não funcionar:** a lupa (o vão continua visível no próprio menisco), depois os grãos caindo durante a execução (fica só o menisco descendo).

## 8. Iteração depois dos screenshots

Screenshots em `design/screenshots/b-*`. Render com o Chromium em cache do Playwright (`chromium-1248`), perfil temporário apagado a cada captura.

- **Visão geral, rodada 1:** a lupa estava poluída (rótulos de graduação, texto e hachura colidindo, e uma elipse inteira desenhando linha tracejada dupla). Refeita: faixa de 7,4 GiB em volta do nível, graduações só à direita, vão preenchido com luz, seta dupla e "3,5 GiB devolvidos", fantasma como uma única curva frontal, leituras "388,7 sem compressão" e "385,2 usados hoje" fora da lente.
- **Visão geral, rodada 2:** a camada "Sistema" lia como vidro vazio; opacidades subidas e Docker tingido com sódio. Descrições das linhas quebravam em duas linhas; textos encurtados e paddings ajustados. A etiqueta "Sistema, apps e documentos" invadia os valores; virou "Sistema e apps".
- **Visão geral, rodada 3:** a lupa e o fio sumiam porque relayouts tardios (troca de fonte) reiniciavam a animação de entrada; relayout silencioso (`is-still`) resolveu. Separadores das linhas com ponta curva (borda de elemento com raio 16) viraram um traço `::before` com recuo de 12 px que some no hover. A 960x640 as leituras da lupa saíam da janela; agora elas descem para baixo da lente quando falta espaço.
- **Alvos, rodada 1:** o fio de luz começava 45 px fora da linha porque a largura do subtítulo mudava quando a fonte carregava depois do posicionamento; `ResizeObserver` + timers + `document.fonts` `loadingdone`.
- **Alvos, rodada 2:** status dos bloqueados passou a dizer o motivo ("Compartilha com 23 venvs", "Muda a cada build") em sódio, e o `aria-label` ganhou "bloqueado".
- **Alvos, rodada 3:** a 960x640 a etiqueta "1,3 no disco" encostava no "Ligado" do painel; colunas compactas passaram para `1fr 168px 244px`. Contraste: status e unidade da linha selecionada usavam `faint` sobre `raised-hi` (4.11:1) e passaram a `muted`.
- **Execuções:** os anéis de progresso apareciam com um quadrado em volta porque a classe `.ring` colidia com o utilitário `ring` do Tailwind v4 (box-shadow de 1 px); renomeada para `.dial`. O SVG do anel do dock ganhou tamanho explícito. Subtítulo do dock passou de `faint` para `muted` (o topo do dock é `raised-hi`).
- **Vazio:** "Ferramentas do mise está desligado" (concordância errada) virou "O mise está desligado".
- **Agendamento:** "(ainda não existe)" truncava a linha do log; virou nota em linha própria. A 360 px o comando e o caminho do log quebram em vez de truncar.
- **360 px:** medido num iframe de 360 px (o Chrome headless não abre janela tão estreita): `scrollWidth` igual à largura em Visão geral, Alvos, Execuções e Agendamento.
- **Auditoria:** `design-audit.mjs` 0 erro e 0 aviso. `contraste.mjs`: todo par usado para texto está acima de 4.5:1; `line` só aparece como borda; `faint` nunca sobre `raised-hi`.
