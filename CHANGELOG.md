# Histórico de versões

## 0.2.2-alpha

Versão candidata em refinamento, sem incremento de versão. Validação funcional e visual manual e aprovação para publicação pendentes.

- Capa sempre visível; modo compacto horizontal com capa/informações à esquerda, reprodução à direita e progresso inferior.
- Ícones e botão de reprodução menores, com áreas clicáveis preservadas; espaçamento vertical reduzido e grupos com larguras máximas.
- Esconder/Fechar e seleções do menu em verde; removida somente a opção de manter a janela no topo.
- Sliders com preenchimento verde e cursor branco circular; popup de volume sem porcentagem numérica.
- Clique no alto-falante alterna o mute das sessões Spotify, preservando os ganhos individuais e sem alterar o volume global.

- Redimensionamento ancorado ao topo durante o arraste, com largura simétrica no alinhamento central e limites do monitor.
- Bandeja persistente; janela fora da barra de tarefas/Alt+Tab; Esconder e Fechar no menu; retorno por duplo clique na bandeja.
- Menu ocupa a própria janela, sem acesso pelo botão direito, e fecha ao clicar fora.
- Controles de reprodução dentro do cartão de progresso, adicionar desabilitado em espaço separado e estado real no cabeçalho.
- Removidos botões tradicionais de minimizar/fechar e sliders de dimensões.
- Modo compacto automático horizontal, mantendo capa proporcional e limite mínimo vertical.
- Textos em uma linha, com rolagem horizontal somente quando excedem a área disponível.
- Controle de volume por sessão de áudio do Spotify, sem controle do volume global.
- Fundo mais preto e capa menor com arredondamento discreto.
- Executável portátil e ZIP; nenhum instalador novo. Checklist manual incluído.

## 0.2.1

- Dropdown completamente oculto, com animação de abertura e recolhimento.
- Detecção do cursor independente do foco e da visibilidade da janela; reapresentação acima de outras janelas sem roubar o foco.
- No dropdown, redimensionamento apenas pelo canto inferior direito; janela livre mantém todas as bordas.
- Checkbox para o modo dropdown e slider do atraso de recolhimento (padrão: 0,8 s). Removidos os atalhos específicos do aplicativo.
- Progresso, tempos e estado de reprodução no mesmo cartão, sem sobrepor a capa ao redimensionar.
- Cabeçalho simplificado, tela principal sem instruções e cantos superiores retos no dropdown.
- Instalador por usuário, desinstalador e versionamento com três números.
- Tela cheia exclusiva e jogos que prendem o cursor continuam sujeitos às restrições do Windows.

## 0.2.0

- Dropdown, janela arrastável e redimensionável, menu de ajustes e aba Sobre.
- Monitor, alinhamento e tamanhos separados por modo, salvos localmente.
- Capa inteira e metadados reorganizados.

## 0.1.0

- Protótipo Rust/egui com metadados e controles reais pela sessão de mídia do Windows, sem OAuth.
