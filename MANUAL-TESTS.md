# Validação manual — v0.2.2 Alpha

**Pendente de execução pelo usuário.** Nesta versão, o Codex apenas revisa o código, verifica a compilação e gera o executável. Nenhum resultado funcional ou visual está aprovado pela compilação.

Extraia o ZIP em uma pasta gravável e abra `Spotify Dropdown.exe`. Use o menu **•••** para os ajustes. As versões anteriores e suas preferências foram preservadas.

## Janela e bandeja

- [ ] Confirme que existe um ícone permanente na bandeja do Windows, inclusive quando a janela está recolhida ou escondida. Verifique também os ícones ocultos da bandeja.
- [ ] Abra/recolha o dropdown várias vezes: nenhum botão do aplicativo deve aparecer na barra de tarefas ou no Alt+Tab.
- [ ] Em **••• → Esconder**, confirme que o aplicativo continua executando, mas o topo da tela não abre o painel. Dê **dois cliques no ícone da bandeja** para reabrir.
- [ ] Em **••• → Fechar**, confirme que a janela e o ícone da bandeja desaparecem e o processo encerra.
- [ ] Arraste o cabeçalho para usar a janela livre; redimensione pelos cantos e confirme a preservação do tamanho ao reabrir.

## Dropdown e redimensionamento

- [ ] Ative o dropdown pelo menu e escolha monitor e alinhamento. Confirme que o painel permanece preso ao topo durante todo o arraste dos cantos inferiores.
- [ ] No alinhamento **Centro**, aumente e diminua a largura: os lados devem se mover simetricamente. Em **Esquerda/Direita**, a borda correspondente deve permanecer ancorada.
- [ ] Altere a altura: a borda superior deve ficar fixa; a inferior acompanha o arraste. A aresta superior não deve redimensionar.
- [ ] Tente ultrapassar os limites mínimo/máximo e as bordas do monitor. Confirme que o painel permanece visível e sem tremulações ou sobreposições.
- [ ] Repita em resoluções e escalas DPI diferentes, e em outro monitor, quando disponíveis.
- [ ] Saia com o cursor: confira o recolhimento com o atraso escolhido, o desaparecimento completo e a reabertura ao encostar no topo na região do painel.
- [ ] Repita sobre outro aplicativo. Tela cheia exclusiva e jogos que prendem o cursor precisam de verificação específica; não há garantia universal nesses cenários.

## Menu e layout

- [ ] Clique em **•••**: a janela inteira deve mostrar as configurações. Clique no **X** para voltar ao controlador. O botão direito não deve abrir um menu.
- [ ] Com o menu aberto, clique fora da janela. O menu deve fechar e o dropdown deve recolher normalmente se o cursor permanecer fora.
- [ ] Teste tamanhos mínimo, intermediário e máximo, com larguras diferentes: anterior, reproduzir/pausar, próxima, adicionar e volume devem permanecer separados e dentro do contêiner.
- [ ] Reduza a altura até o modo compacto: a capa deve sumir e os controles essenciais permanecer acessíveis. Expanda para a capa reaparecer; varie lentamente perto do limite para verificar estabilidade.
- [ ] Use faixas com nomes longos de música, artista e álbum: cada texto deve ocupar uma linha, rolar suavemente para a direita apenas quando necessário e permanecer recortado à própria área.
- [ ] Confirme fundo preto, capa menor com arredondamento discreto e sem distorção, ausência dos antigos sliders de dimensões e dos botões tradicionais de minimizar/fechar.

## Spotify e volume

- [ ] Com Spotify Desktop reproduzindo neste computador, confira faixa, artista, álbum, capa e tempos. Verifique anterior, reproduzir/pausar e próxima.
- [ ] Confirme que o cabeçalho mostra círculo verde + **REPRODUZINDO**, ou duas barras verdes + **PAUSADO**, conforme o estado real, sem indicador duplicado junto ao progresso.
- [ ] Passe o cursor sobre volume: o painel vertical deve abrir junto ao botão. Ajuste de 0% a 100% e confira o volume do Spotify no Mixer do Windows.
- [ ] Enquanto ajusta o Spotify, confira que o volume global e o áudio de outro aplicativo permanecem inalterados. Teste também sessões/saídas de áudio diferentes, quando disponíveis.
- [ ] Mova o cursor entre botão e painel: ele deve permanecer aberto. Saia de ambos: ele deve desaparecer sem ultrapassar a janela ou o monitor, inclusive no modo compacto.
- [ ] Feche o Spotify ou deixe-o sem sessão de áudio; confira que controles indisponíveis são desabilitados e o aplicativo continua responsivo. Reabra o Spotify e verifique a reconexão.
- [ ] O botão de adicionar à biblioteca e playlists devem continuar desabilitados, sem simular integração.

Ao relatar um problema, informe tamanho aproximado da janela, modo, alinhamento, monitor/escala DPI e sequência de ações. Capturas e gravações podem ser feitas manualmente pelo usuário.
