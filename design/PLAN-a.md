---
title: Plano de design - Disk Steward, direção A (Prensa de vidro)
status: proposta em arena
fonte-de-verdade: docs/API.md (contrato Overview / Target / Preview / RunResult / ScheduleAgent / DockerUsage)
referencia-de-acabamento: attack-shark-x3 (design/PLAN.md, docs/images, design/screenshots/a-*)
---

# Plano de design — Disk Steward, direção A: "Prensa de vidro"

> Preenchido antes da primeira linha do protótipo e revisado depois de cada rodada de screenshots
> (seção 8). Esta é só a direção A; a B é escrita em paralelo em `PLAN-b.md`. Depois da escolha do
> Lucas, o vencedor é fundido em `design/PLAN.md` e `design/tokens.css`.

## 0. Projeto

- **Stack e versões (alvo):** Tauri v2 + Vue 3 (`<script setup lang="ts">`) + Vite + Tailwind CSS v4 (CSS-first, sem `tailwind.config.js`) + `motion-v` + `vue-i18n` + Pinia. O único backend é `crates/steward-core` (Rust), que chama o CLI `auto-compress-apfs`. Nenhuma camada web existe ainda.
- **Design system / tokens existentes:** nenhum. O que existe é o contrato de `docs/API.md`; o protótipo usa os mesmos nomes de campo (`freeBytes`, `privateBytes`, `risk`, `riskDetail`, `freedBytes`, `lastExitStatus`...), então a UI real troca o mock por `invoke()` sem renomear nada.
- **Componentes que já resolvem parte disto:** nenhum componente. Herda-se da referência attack-shark-x3 o **nível de acabamento e o esqueleto de janela** (sidebar com pílula elevada, linhas com ícone em círculo, dock persistente com a ação única), não a paleta, a fonte nem o objeto.
- **Convenções a seguir:** código, tokens e identificadores em inglês; texto de interface em pt-BR com acento correto, via `vue-i18n`. Números com `Intl.NumberFormat('pt-BR')`. **Unidades binárias (GiB, MiB)**, as mesmas do `df -h` e do motor; o Finder mostra GB decimais e por isso diverge, o que é dito uma vez, na tela de Alvos. Loading e erro aparecem onde o dado aparece; o dock é a superfície única do que está pendente.
- **Restrições da stack:** fonte self-hosted via `@fontsource-variable/geologica` (OFL), sem CDN no app. Janela com `titleBarStyle: Overlay` (semáforos sobre a sidebar), padrão 1180x760, mínimo 960x640. O protótipo usa `@tailwindcss/browser@4.3.5`, `motion@14.14.0` (ESM do jsDelivr) e a Geologica do Google Fonts **só no protótipo**.

## 1. Assunto

- **O que é, concretamente:** o utilitário de barra de menus de um MacBook M4 Pro com SSD interno de 460,4 GiB e 75,2 GiB livres. Mostra para onde vai o disco de desenvolvimento, mantém caches e toolchains comprimidos com a compressão transparente do APFS, **prova** o ganho em bytes liberados medidos pelo `df` e **segura** as ações que sairiam pela culatra.
- **Para quem:** o Lucas, dev sênior. Sabe o que é clone APFS, `du`, `df` e launchd. Não quer tutorial; quer números honestos e um freio quando o ganho aparente é falso.
- **O único trabalho desta tela:** mostrar a verdade do tamanho. Três tamanhos por pasta: **lógico** (o que o `du` soma), **no disco** (blocos ocupados) e **privado** (bytes que nenhum clone divide). Com clones, o `du` mente; o app desenha os três no mesmo objeto.
- **Vocabulário do assunto:** volume, bloco, container APFS, clone (`clonefile`), compressão transparente (decmpfs), conferência byte a byte, troca atômica, quarentena, `df`, `du`, statfs, agente do launchd, código de saída, venv, store, toolchain.

## 2. Cor

Cinco matizes. Neutros vêm do alumínio do próprio Mac; as três luzes de estado vêm do LED do MagSafe (verde = cheio, âmbar = carregando) e do semáforo vermelho da janela do macOS. A cor nunca é decoração: ela é a **luz que o bloco emite**, e só muda quando o estado muda.

| Token | Hex | De onde vem |
| --- | --- | --- |
| `--color-aluminum` | `#E6E8EA` | O alumínio prata do MacBook Pro à luz do dia. Chão da janela. Neutro frio, nunca creme. |
| `--color-anodized` | `#1F2327` | O alumínio anodizado "preto-espacial". Texto, ícones e a única ação primária. |
| `--color-magsafe-green` | `#2FBF71` | O LED do MagSafe com a bateria cheia. Luz de "comprimido e conferido". Texto usa `#13703F`. |
| `--color-magsafe-amber` | `#F5A524` | O LED do MagSafe carregando. Luz de "trabalho em andamento" (pré-visualização, execução). Texto usa `#7D5206`. |
| `--color-signal-red` | `#E5484D` | O botão vermelho do semáforo da janela. Luz de falha, nunca de bloqueio. Texto usa `#B3261E`. |
| `--color-quartz` | `#A9BDC9` | Vidro com a luz apagada, frio como quartzo. Alvo bloqueado: calmo, sem alarme. Texto usa `#3D5664`. |

Derivados (não contam como matiz): `--color-frost` `#F8F9FA` (vidro jateado: superfícies elevadas), `--color-aluminum-deep` `#D0D4D8` (fios e trilhos vazios), `--color-anodized-muted` `#555B62` (texto secundário). O quartzo é o sexto token, mas é o próprio vidro do objeto sem luz, não uma cor de marca.

Contraste verificado com `scripts/contraste.mjs` (par a par, porque os dois temas usam os mesmos nomes):
- corpo `#1F2327` sobre `#E6E8EA`: **12.87:1**; sobre `#F8F9FA`: **15.00:1**.
- secundário `#555B62` sobre `#E6E8EA`: **5.59:1**; sobre `#F8F9FA`: **6.51:1**.
- botão `#F8F9FA` sobre `#1F2327`: **15.00:1**.
- verde texto `#13703F` sobre `#F8F9FA`: **5.83:1**; sobre `#E6E8EA`: **5.00:1**.
- âmbar texto `#7D5206` sobre `#F8F9FA`: **6.46:1**. Vermelho texto `#B3261E` sobre `#F8F9FA`: **6.20:1**.
- quartzo texto `#3D5664` sobre `#F8F9FA`: **7.34:1**; sobre `#E6E8EA`: **6.30:1**.
- Tema escuro ("noite", mesmas origens): texto `#E9EBED` sobre `#16191C` **14.77:1**; secundário `#A1A8AF` sobre `#1F2429` **6.51:1**; verde `#5BD996` **8.80:1**; âmbar `#F2B451` **8.51:1**; vermelho `#FF8F8A` **7.11:1**; quartzo `#A9C3D2` **8.51:1**; botão `#16191C` sobre `#E9EBED` **14.77:1**.
- As luzes puras (`#2FBF71`, `#F5A524`, `#E5484D`) nunca carregam texto: são preenchimento do objeto, poça no chão e pontos de estado com rótulo ao lado.

## 3. Tipo

- **Geologica (OFL, variável em peso 100..900, mais os eixos `SHRP`, `CRSV` e `slnt`).** Uma família só. Escolhida porque os pesos 100..250 dão numerais enormes finos e calmos (o "75,2" da Visão geral em peso 200 lê como instrumento, não como banner) e porque o desenho é uma grotesca com terminais retos e aberturas largas, que segura legenda de 12 px sem apertar. Não é a grotesca de inércia (Inter, Geist, SF), não é a Mona Sans da referência, e o nome não é acaso: o assunto é volume, camada, bloco.
- **Papéis:** numeral de herói (peso 200, uma vez por tela), título de tela (peso 300), rótulo (400), valor à direita das linhas (peso 350, `tabular-nums`), legenda (400, 12 px). Distinção por tamanho e peso, nunca por caixa alta. Caminhos (`~/.cargo/registry`) ficam na mesma família; só as linhas de log do motor usam o mono do sistema (`ui-monospace`), porque ali o mono é função (é saída de terminal), não enfeite.
- **Escala:** terça menor (1.2) a partir de 14 px: 12 / 14 / 16,8 / 20,2 / 24,2 / 29 / 41,8 / 60,2 / 86,7. Os três maiores em `clamp()` no `@theme` (`--text-display`, `--text-numeral`, `--text-hero`). Medida das explicações limitada a 56ch.

## 4. Composição

### Arquitetura de informação (igual à da outra direção)

Sidebar com quatro destinos, do "como está" para o "o que automatiza": **Visão geral**, **Alvos**, **Execuções**, **Agendamento**. `Cmd+1..4` navega, `Cmd+Return` dispara a ação do dock. A barra de menus é a porta rápida; a janela é a conferência.

- **Dock persistente** no rodapé do conteúdo, em todas as telas: diz o que está pendente em linguagem humana e guarda a **única ação primária**. Estados: ocioso (arquivos novos podem ter chegado), pré-visualizando, pré-visualização pronta, execução em andamento (com "parar depois deste arquivo"), concluída (bytes do `df`), falhou (original intacto, tentar de novo), medindo, motor não encontrado, nenhum alvo ligado.
- **Bloqueado é calmo:** o interruptor vira um cadeado, a luz do bloco se apaga em quartzo, a prensa fica presa em dois batentes e o painel diz em uma frase por que e **o que fazer para liberar** (apagar os venvs antigos, `UV_LINK_MODE=symlink`).
- **Avisar antes do tiro no pé:** comprimir sempre passa por pré-visualização e confirmação; a pré-visualização é rotulada como estimativa; o resultado só mostra número que veio do `df`.

### Wireframes

Visão geral (1180x760):
```
+--------------+-----------------------------------------------------------------+
| o o o        |                                                                 |
| [] Disk      |        poça de luz verde no chão                                |
|    Steward   |          .------------.            75,2  GiB livres             |
|  medido 15:42|         /  vidro vazio  \          no Macintosh HD, de 460,4    |
|              |        |---------------- |         ------------------------    |
| (Visão geral)|        |  luz verde:     |         (o) Em uso        385,2 GiB  |
|  Alvos       |        |  385,2 em uso   |             [segmentos ======--]    |
|  Execuções   |        |                 |         (o) Devolvido       3,5 GiB |
|  Agendamento |         \_______________/          (o) Docker         46,9 GiB |
|              |          sombra + cáustica         (o) Parados         3 alvos |
| Livre 75,2   |                                    [cubo rustup -58%][cargo..] |
| [Macintosh HD]| [ dock: arquivos novos desde 21 nov ...       (Pré-visualizar)] |
+--------------+-----------------------------------------------------------------+
  sidebar 232 | grade 5fr/6fr, texto à esquerda; o bloco é o volume desenhado como volume
```

Alvos (lista à esquerda, painel com o bloco do alvo à direita, como a tela de DPI da referência):
```
| sidebar | Alvos                                                  [Medir de novo] |
|         | Três tamanhos por pasta, porque com clones o du conta errado.          |
|         | Ligados                          +-- painel elevado ------------------+ |
|         | [cubo] Toolchains do Rust 1,3 [on]| Toolchains do Rust      Ligado    | |
|         | [cubo] Registro do Cargo  1,1 [on]| [bloco ]   1,3  GiB no disco      | |
|         | [cubo] Cache do npm       1,1 [on]| [+ghost]   de 3,1 lógicos         | |
|         | Podem ser ligados                 | --- Lógico  o que o du soma  3,1  | |
|         | [cubo] Ferramentas do mise 1,4 [ ]| --- No disco blocos ocupados 1,3  | |
|         | [cubo] Store do pnpm      2,9 [ ] | --- Privado  só desta pasta  1,3  | |
|         | Ficam de fora                     | 9.512 de 9.840 arquivos [=====-]  | |
|         | [cubo] Cache do uv        7,4 [L] | [Comprimir esta pasta  (o)] Finder| |
|         | [cubo] Store do Bun       4,2 [L] +-----------------------------------+ |
|         | [cubo] Pastas target   --     [L]                                       |
|         | [ dock ]                                                                |
```

Execuções (o bloco da execução à esquerda, a fase à direita):
```
| sidebar | Execuções                                                               |
|         | Pré-visualizar, comprimir e provar.                                     |
|         |   prensa  ____                       421  MiB a comprimir               |
|         |   [bloco da execução]                1.312 arquivos em 3 pastas         |
|         |   desce com a luz âmbar              (o) ~/.cargo/registry   212 MiB    |
|         |   assenta verde no fim               (o) ~/.npm              148 MiB    |
|         |   ghost tracejado = antes            (o) ~/.rustup            61 MiB    |
|         | [ dock: 1.312 arquivos prontos   (Descartar) (Comprimir 421 MiB...) ]   |
  fases: ocioso (última execução e histórico) / pré-visualizando / pré-visualização /
  confirmação (diálogo) / comprimindo (contador e pastas) / concluída (df antes e depois) /
  falhou (log do motor, original intacto, tentar de novo)
```

Agendamento:
```
| sidebar | Agendamento                                                            |
|         | Os agentes do launchd que trabalham sem abrir o app.                   |
|         |   [bloco, prensa erguida]   (o) Compressão diária          Desligado   |
|         |                                 local.auto-compress-apfs [Instalar]   |
|         |                             (o) Limpeza do Docker          Carregado   |
|         |                                 com.lucasbrito.docker-gc  saída 0     |
```

Barra de menus (320 px): cubo aceso + `75,2 GiB livres`, linha de devolvido, estado da última execução (ou progresso, ou falha), `[Pré-visualizar compressão]`, `Abrir Disk Steward`, `Medir de novo`, `Sair`.

- **Quebra estrutural escolhida:** um objeto físico único, o bloco de vidro, que atravessa as telas (herói da Visão geral, bloco do alvo dentro do painel de Alvos, prensa da Execução, prensa parada no Agendamento, cubo em miniatura na sidebar, na lista e na barra de menus). Pertence ao assunto porque "volume" é literalmente uma grandeza de três dimensões: o app desenha o volume como volume.
- **Alinhamento:** à esquerda. Centralizado só o objeto dentro da sua coluna.
- **O que fica fora do grid:** o bloco e a poça de luz, que se desloca com ele e tinge o chão da janela.

## 5. Assinatura

- **O mecanismo, em uma frase:** **a prensa de vidro** — todo tamanho é um bloco de vidro cuja altura é o tamanho no disco, com o contorno tracejado da altura lógica acima dele, a parte privada acesa por dentro e a parte compartilhada por clone como vidro laminado com fios que saem para os blocos que a dividem (os 23 venvs); comprimir é a placa de alumínio descendo e o bloco encolhendo até a altura nova, deixando o fantasma tracejado da altura antiga.
- **Manifestações:** Visão geral (o disco inteiro: luz = em uso, vidro vazio = livre); Alvos (o bloco do alvo; bloqueado = luz apagada, laminado, prensa presa em batentes, cadeado); Execuções (a prensa desce durante a execução com luz âmbar pulsando, assenta verde ao fim; na falha, para no meio com a costura vermelha); Agendamento (prensa erguida esperando); medindo (só o contorno, com um plano de luz que sobe e desce); barra de menus e sidebar (cubo em miniatura com a luz do estado).
- **Curva de easing de assinatura:** `--ease-platen: cubic-bezier(0.5, 0, 0.1, 1)` — a placa de uma prensa: aproximação firme, assentamento sem quique. A mesma curva no CSS (`ease-platen`) e no Motion (`[0.5, 0, 0.1, 1]`). `prefers-reduced-motion` zera as durações e desliga o pulso e o plano de varredura; o estado final continua igual.
- **Por que isto não existe em outro produto:** analisadores de disco mostram **um** tamanho por pasta, em mapa de árvore (GrandPerspective), anel (DaisyDisk) ou lista (Finder); utilitários de limpeza usam ilustração 3D como enfeite. Aqui a geometria **é** o dado: alturas em bytes, o fantasma é o `du`, a luz é o privado, os fios são os clones, e o objeto só se move quando o disco muda.

## 6. Revisão contra os clichês

1. **Creme quente + serifa + terracota:** não. Chão de alumínio frio `#E6E8EA`, uma grotesca, nenhuma cor quente fora do âmbar de estado.
2. **Quase preto + acento ácido ou vermelho:** não no padrão (claro). O tema escuro é derivado do alumínio anodizado `#16191C` com as mesmas três luzes de estado; vermelho só para falha.
3. **Jornal, fios finos, zero radius:** não. Raios por hierarquia (tecla 10 px, painel e dock 20 px, círculos de ícone), fios só entre linhas de leitura.
4. **Kit de cards de SaaS:** evitado. A Visão geral é um objeto e quatro linhas sobre o chão; só existe um painel elevado (o detalhe do alvo) e o dock. Sombras tingidas com o anodizado, nunca `rgba(0,0,0,.1)`.
5. **Cromo de template:** sem eyebrow em caixa alta, sem "A · B · C", sem seta em botão, sem preto tintado, sem mono decorativo (mono só nas linhas de log). Os rótulos de grupo da lista ("Ligados", "Podem ser ligados", "Ficam de fora") codificam o estado real do alvo.

**O que mudou depois desta revisão:**
- (a) Comecei pelo exemplo do brief, o módulo SSD com pacotes NAND que acendem por ocupação. Descartei: um chip não mostra **três** tamanhos, e todo o produto existe por causa do terceiro (privado). O bloco de vidro mostra os três sem legenda.
- (b) A segunda ideia foi um tanque com nível de líquido. Descartei: líquido sugere fluxo e vazamento, e comprimir não é escoar, é prensar.
- (c) A luz de bloqueio começou âmbar. Virou luz **apagada** (quartzo): âmbar no MagSafe quer dizer "carregando", e bloqueio não é trabalho em andamento nem perigo; é a ausência deliberada de ação.
- (d) A versão rejeitada pelo Lucas (card escuro com barras vazias e números com espaço) é exatamente o kit de cards do clichê 4. Aqui não existe grade de cards, todo número passa por `Intl.NumberFormat('pt-BR')` e é conferido no screenshot, e o protótipo não tem barra de debug dentro da janela (controles por hash e teclado, painel flutuante só fora da janela).

## 7. Riscos

- **A aposta mais ousada:** um objeto 3D desenhado (SVG com faces, especular, sombra de contato, cáustica e poça de luz) num utilitário de dev, onde o padrão é tabela ou terminal.
- **O que a torna defensável:** o objeto é o gráfico. Ele substitui três barras e uma legenda, e é o único lugar onde "privado" e "compartilhado por clone" ficam óbvios de relance. É barato: polígonos recalculados por uma função de projeção isométrica, quatro filtros de blur estáticos.
- **O que eu corto primeiro se não funcionar:** a cáustica e o especular (o bloco continua legível com faces e luz), depois a viagem do objeto entre telas (cada tela desenha o seu no lugar).
- **Dado a confirmar com o motor:** o total de `savings` (3,5 GiB) não fecha exatamente com a soma dos alvos ligados (3,0 GiB): o motor arredonda a uma casa e conta padrões além do catálogo. O app mostra cada número com sua fonte e não tenta reconciliar.

## 8. Iteração depois dos screenshots

Captura com o Chromium do cache do Playwright (`chromium-1248`), escala 2x, janela 1180x760 e 960x640, além de 360 px por emulação de viewport. A primeira rodada usou `--headless=new --virtual-time-budget`; a rodada final passou para captura em tempo real por CDP, porque o tempo virtual congelava a transição de `--hl` do objeto (o bloco ficava verde em "Comprimindo" quando o certo é âmbar). Mais de três iterações em Visão geral e Alvos.

- **Visão geral:** (1) cubos pretos nos ícones: o build de navegador do Tailwind v4 só emite as variáveis de tema usadas por utilitários, e as cores lidas por `var()` dentro do SVG caíam no valor inicial; resolvido com `@theme static`. (2) A coluna do objeto crescia até o min-content do slot de altura fixa e espremia os números; virou `grid-cols-[minmax(0,4fr)_minmax(0,6fr)]` com o slot dimensionado pela largura. (3) O numeral gigante aparecia como "7 4" porque os algarismos tabulares abrem espaço no "1" e no "7"; os numerais de destaque usam `proportional-nums`, as colunas de valores mantêm `tabular-nums`. (4) Legenda "luz: dados em uso / vidro claro: espaço livre" sob o objeto, escondida durante a medição. (5) Abaixo de 44rem o texto vem antes do objeto (`order-first`). (6) Em "Medindo" e "Sem motor" a nota da linha "Devolvido pela compressão" contradizia o valor; agora diz de onde o número viria.
- **Alvos:** (1) O painel de detalhe ganhou `@container/panel` com o slot do objeto em 164 px, que cai para 112 px abaixo de 22rem. (2) A 960 px o título do painel e as notas das três medidas eram truncados com reticências; o título quebra linha e a nota desce para baixo do rótulo no painel estreito. (3) Em alvo bloqueado, a caixa de risco vem antes das medidas: o motivo do bloqueio é a informação principal. (4) As linhas da lista ficaram 4 px mais baixas para os 8 alvos caberem acima do dock a 1180x760.
- **Execuções:** (1) O diálogo de confirmação não aparecia: a animação WAAPI do Motion não roda em tempo virtual; a entrada do diálogo e dos toasts passou para keyframe CSS com a mesma `--ease-platen`. (2) Na falha, a costura vermelha ficava escondida sob a placa; a placa agora para 34 px acima do bloco. (3) Linhas de log usam `overflow-wrap:anywhere` no lugar de `break-all`, que cortava palavras comuns.
- **Barra de menus:** durante a execução e na falha, o objeto do menu mostra a mesma prensa da janela (`trayHero()` reaproveita `heroForRun()`).
- **Auditoria:** `design-audit.mjs` em `claude-a.html` e `tokens-a.css`: 0 erro e 0 aviso, depois de trocar o anel `rgb(0 0 0 / 0.08)` da moldura da janela por anodizado `rgb(31 35 39 / 0.12)` e os meios-passos de espaçamento (6 e 10 px) pela escala de 4 px.
- **Contraste (WCAG, `contraste.mjs` par a par):** claro, sobre alumínio `#E6E8EA` e frost `#F8F9FA`: anodizado 12,87 e 15,00; anodizado gasto 5,59 e 6,51; verde 5,00 e 5,83; âmbar 5,55 e 6,46; vermelho 5,32 e 6,20; quartzo 6,30 e 7,34. Escuro, sobre `#16191C` e `#1F2429`: o pior par é o anodizado gasto `#A1A8AF` sobre frost, com 6,51. Botão primário: 15,00 (claro) e 13,09 (escuro). Todos AA para texto normal.
- **360 px:** sem rolagem horizontal em Visão geral, Alvos (ligado e bloqueado), Execuções (comprimindo e falhou), Agendamento e barra de menus: `scrollWidth` do documento é 360 e nenhum elemento rolável transborda. A sidebar vira uma faixa de abas no topo e o dock esconde a segunda linha.
- **O que ainda fica aquém da referência:** o objeto é um SVG isométrico, sem a profundidade de material de um render 3D de verdade; a 760 px de altura o painel de detalhe do alvo e a tela de falha rolam por baixo do dock; os medidores segmentados viram pontos em tamanhos pequenos; os 3,5 GiB de `savings` não fecham com os 3,0 GiB somados dos alvos (seção 7); o fundo da cena da barra de menus é um gradiente liso, não um papel de parede.
