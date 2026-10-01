<script lang="ts">
  let { os = '', name = '', tags = [], address = '', size = 16, class: className = '' }: {
    os?: string;
    name?: string;
    tags?: string[];
    address?: string;
    size?: number;
    class?: string;
  } = $props();

  export function detectOsKey(explicitOs?: string, hostName?: string, hostTags?: string[], hostAddress?: string): string {
    const raw = (explicitOs || '').trim().toLowerCase();
    if (raw) {
      // *buntu flavors contain "ubuntu" as a substring, so they must be checked first.
      if (raw.includes('kubuntu')) return 'kubuntu';
      if (raw.includes('xubuntu')) return 'xubuntu';
      if (raw.includes('lubuntu')) return 'lubuntu';
      if (raw.includes('ubuntu')) return 'ubuntu';
      if (raw.includes('debian')) return 'debian';
      if (raw.includes('fedora')) return 'fedora';
      if (raw.includes('red hat') || raw.includes('redhat') || raw.includes('rhel')) return 'redhat';
      if (raw.includes('centos')) return 'centos';
      if (raw.includes('rocky')) return 'rocky';
      if (raw.includes('almalinux') || raw.includes('alma')) return 'almalinux';
      if (raw.includes('arch')) return 'arch';
      if (raw.includes('manjaro')) return 'manjaro';
      if (raw.includes('alpine')) return 'alpine';
      if (raw.includes('opensuse') || raw.includes('suse')) return 'opensuse';
      if (raw.includes('mint')) return 'mint';
      if (raw.includes('kali')) return 'kali';
      if (raw.includes('pop') || raw.includes('pop!_os') || raw.includes('popos')) return 'popos';
      if (raw.includes('gentoo')) return 'gentoo';
      if (raw.includes('void')) return 'void';
      if (raw.includes('nixos') || raw.includes('nix')) return 'nixos';
      if (raw.includes('endeavour')) return 'endeavour';
      if (raw.includes('elementary')) return 'elementary';
      if (raw.includes('zorin')) return 'zorin';
      if (raw.includes('raspbian') || raw.includes('raspberry') || raw.includes('rpi')) return 'raspberry';
      if (raw.includes('amazon') || raw.includes('amzn')) return 'amazon';
      if (raw.includes('oracle')) return 'oracle';
      if (raw.includes('slackware')) return 'slackware';
      if (raw.includes('mageia')) return 'mageia';
      if (raw.includes('solus')) return 'solus';
      if (raw.includes('tails')) return 'tails';
      if (raw.includes('deepin')) return 'deepin';
      if (raw.includes('clear')) return 'clear';
      if (raw.includes('garuda')) return 'garuda';
      if (raw.includes('steamos') || raw.includes('steam')) return 'steam';
      if (raw.includes('coreos')) return 'coreos';
      if (raw.includes('flatcar')) return 'flatcar';
      if (raw.includes('devuan')) return 'devuan';
      if (raw.includes('parrot')) return 'parrot';
      if (raw.includes('mx')) return 'mx';
      if (raw.includes('freebsd')) return 'freebsd';
      if (raw.includes('openbsd')) return 'openbsd';
      if (raw.includes('netbsd')) return 'netbsd';
      if (raw.includes('mikrotik') || raw.includes('routeros')) return 'mikrotik';
      if (raw.includes('cisco')) return 'cisco';
      if (raw.includes('windows') || raw.includes('win')) return 'windows';
      if (raw.includes('macos') || raw.includes('apple') || raw.includes('mac') || raw.includes('darwin')) return 'macos';
      if (raw.includes('android')) return 'android';
      if (raw.includes('ios')) return 'ios';
      if (raw.includes('server')) return 'server';
      if (raw.includes('linux')) return 'linux';
      return raw;
    }

    const corpus = `${hostName || ''} ${(hostTags || []).join(' ')} ${hostAddress || ''}`.toLowerCase();
    // Same ordering requirement as above: *buntu flavors before the plain "ubuntu" check.
    if (corpus.includes('kubuntu')) return 'kubuntu';
    if (corpus.includes('xubuntu')) return 'xubuntu';
    if (corpus.includes('lubuntu')) return 'lubuntu';
    if (corpus.includes('ubuntu')) return 'ubuntu';
    if (corpus.includes('debian')) return 'debian';
    if (corpus.includes('fedora') || corpus.includes('fc4')) return 'fedora';
    if (corpus.includes('rhel') || corpus.includes('redhat') || corpus.includes('red hat')) return 'redhat';
    if (corpus.includes('centos')) return 'centos';
    if (corpus.includes('rocky')) return 'rocky';
    if (corpus.includes('alma') || corpus.includes('almalinux')) return 'almalinux';
    if (corpus.includes('arch')) return 'arch';
    if (corpus.includes('manjaro')) return 'manjaro';
    if (corpus.includes('alpine')) return 'alpine';
    if (corpus.includes('opensuse') || corpus.includes('suse')) return 'opensuse';
    if (corpus.includes('mint')) return 'mint';
    if (corpus.includes('kali')) return 'kali';
    if (corpus.includes('pop-os') || corpus.includes('popos') || corpus.includes('pop!_os')) return 'popos';
    if (corpus.includes('gentoo')) return 'gentoo';
    if (corpus.includes('void')) return 'void';
    if (corpus.includes('nixos') || corpus.includes('nix')) return 'nixos';
    if (corpus.includes('endeavour')) return 'endeavour';
    if (corpus.includes('elementary')) return 'elementary';
    if (corpus.includes('zorin')) return 'zorin';
    if (corpus.includes('raspbian') || corpus.includes('raspberry') || corpus.includes('rpi')) return 'raspberry';
    if (corpus.includes('amazon') || corpus.includes('amzn') || corpus.includes('ec2') || corpus.includes('aws')) return 'amazon';
    if (corpus.includes('oracle') || corpus.includes('oci')) return 'oracle';
    if (corpus.includes('slackware')) return 'slackware';
    if (corpus.includes('mageia')) return 'mageia';
    if (corpus.includes('solus')) return 'solus';
    if (corpus.includes('tails')) return 'tails';
    if (corpus.includes('deepin')) return 'deepin';
    if (corpus.includes('clear linux') || corpus.includes('clearlinux')) return 'clear';
    if (corpus.includes('garuda')) return 'garuda';
    if (corpus.includes('steam')) return 'steam';
    if (corpus.includes('flatcar')) return 'flatcar';
    if (corpus.includes('coreos')) return 'coreos';
    if (corpus.includes('devuan')) return 'devuan';
    if (corpus.includes('parrot')) return 'parrot';
    if (corpus.includes('mx linux') || corpus.includes('mxlinux')) return 'mx';
    if (corpus.includes('freebsd') || corpus.includes('bsd')) return 'freebsd';
    if (corpus.includes('openbsd')) return 'openbsd';
    if (corpus.includes('netbsd')) return 'netbsd';
    if (corpus.includes('mikrotik') || corpus.includes('routeros')) return 'mikrotik';
    if (corpus.includes('cisco') || corpus.includes('ios-xe')) return 'cisco';
    if (corpus.includes('windows') || corpus.includes('win10') || corpus.includes('win11') || corpus.includes('winserver') || corpus.includes('rdp')) return 'windows';
    if (corpus.includes('mac') || corpus.includes('osx') || corpus.includes('apple') || corpus.includes('darwin')) return 'macos';
    if (corpus.includes('android')) return 'android';
    if (corpus.includes('ios') || corpus.includes('iphone') || corpus.includes('ipad')) return 'ios';
    if (corpus.includes('vps') || corpus.includes('srv') || corpus.includes('node') || corpus.includes('host') || corpus.includes('server')) return 'server';

    return 'linux';
  }

  let osKey = $derived(detectOsKey(os, name, tags, address));

  // Design system for every badge below: a filled r=10 circle in the distro's real brand
  // color, plus a glyph simple and bold enough to still read at the 16-22px this actually
  // renders at (host cards, badges, the add-host form). Icons people recognize by shape keep
  // a hand-drawn simplification of the real mark (Ubuntu's dots, Debian's swirl, Arch's
  // triangle, Tux, Windows' panes, Apple, Android...); everything more obscure gets a clean
  // monogram in its real accent color instead of an invented squiggle that resembled nothing.
</script>

<span
  class="inline-flex items-center justify-center shrink-0 {className}"
  style="width: {size}px; height: {size}px;"
  title={osKey}
>
  {#if osKey === 'ubuntu'}
    <!-- Ubuntu "circle of friends" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#E95420" />
      <circle cx="6.5" cy="12" r="1.7" fill="#ffffff" />
      <circle cx="14.8" cy="7.2" r="1.7" fill="#ffffff" />
      <circle cx="14.8" cy="16.8" r="1.7" fill="#ffffff" />
      <path d="M12 5.2a6.8 6.8 0 0 1 5 2.2l-1.3 1.3a5 5 0 0 0-3.7-1.7 5.1 5.1 0 0 0-4.6 3h-1.9a7 7 0 0 1 6.5-4.8zm0 13.6a7 7 0 0 1-6.5-4.8h1.9a5.1 5.1 0 0 0 4.6 3 5 5 0 0 0 3.7-1.7l1.3 1.3a6.8 6.8 0 0 1-5 2.2zm4.3-8.8a5 5 0 0 0 0 4h1.9a6.9 6.9 0 0 1 0-4h-1.9z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'debian'}
    <!-- Debian swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#A81D33" />
      <path d="M15.2 6.3c-3.7 0-6.9 2.8-7.3 6.5-.4 3.3 1.6 6.4 4.8 7.5" fill="none" stroke="#ffffff" stroke-width="2.1" stroke-linecap="round" />
      <circle cx="15.7" cy="6.5" r="1.3" fill="#ffffff" />
    </svg>
  {:else if osKey === 'fedora'}
    <!-- Fedora "f" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#294172" />
      <path d="M10 17.5V9.8a3.3 3.3 0 0 1 3.3-3.3h1.2" fill="none" stroke="#ffffff" stroke-width="2.1" stroke-linecap="round" />
      <path d="M9.7 13h4.3" stroke="#ffffff" stroke-width="2.1" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'redhat'}
    <!-- Red Hat fedora-hat emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#EE0000" />
      <path d="M7 15.3c0-3.4 2.2-6.1 5-6.1s5 2.7 5 6.1H7z" fill="#ffffff" />
      <ellipse cx="12" cy="15.4" rx="7.3" ry="1.7" fill="#ffffff" />
      <path d="M9.3 9.4c.6-1.2 1.7-2 2.9-2.1" stroke="#EE0000" stroke-width="1" fill="none" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'centos'}
    <!-- CentOS 4-quadrant symbol -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <path d="M12 2.2l4.6 4.6H7.4L12 2.2z" fill="#93227F" />
      <path d="M21.8 12l-4.6 4.6V7.4L21.8 12z" fill="#EFA724" />
      <path d="M12 21.8l-4.6-4.6h9.2L12 21.8z" fill="#8EB737" />
      <path d="M2.2 12l4.6-4.6v9.2L2.2 12z" fill="#262577" />
      <rect x="9.3" y="9.3" width="5.4" height="5.4" fill="#ffffff" />
    </svg>
  {:else if osKey === 'rocky'}
    <!-- Rocky Linux mountain peak -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#10B981" />
      <path d="M7 16.5l3.6-6.3a1.6 1.6 0 0 1 2.8 0l3.6 6.3H7z" fill="#ffffff" />
      <circle cx="12" cy="8.2" r="1.2" fill="#ffffff" />
    </svg>
  {:else if osKey === 'almalinux'}
    <!-- AlmaLinux "A" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#15283d" />
      <path d="M12 6.3l5 11.4h-2.3l-1-2.3h-3.4l-1 2.3H7l5-11.4zm0 3.7l-1.3 3h2.6L12 10z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'arch'}
    <!-- Arch Linux -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1793D1" />
      <path d="M12 5.8l3.4 7.7-2.1-1-1.3 3.1-1.3-3.1-2.1 1L12 5.8z" fill="#ffffff" />
      <path d="M7.3 18.2c1.5-1 3-1.5 4.7-1.5s3.2.5 4.7 1.5" fill="none" stroke="#ffffff" stroke-width="1.3" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'manjaro'}
    <!-- Manjaro 3 bars -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#151915" />
      <rect x="6" y="6" width="3.1" height="12" rx="0.5" fill="#35BF5C" />
      <rect x="10.5" y="6" width="3.1" height="12" rx="0.5" fill="#35BF5C" />
      <rect x="15" y="10.2" width="3.1" height="7.8" rx="0.5" fill="#35BF5C" />
    </svg>
  {:else if osKey === 'alpine'}
    <!-- Alpine Linux mountain range -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0D597F" />
      <path d="M4.5 17l4.3-7.8 2 3.6 1.2-2.2 1.2 2.2 2-3.6L19.5 17H4.5z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'opensuse'}
    <!-- openSUSE Geeko -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#73BA25" />
      <path d="M8 16c-1.7-1-2.3-3-1.4-4.7A3.6 3.6 0 0 1 11 9.5" fill="none" stroke="#ffffff" stroke-width="1.7" stroke-linecap="round" />
      <circle cx="14.4" cy="9" r="3.1" fill="#ffffff" />
      <circle cx="15.5" cy="8.1" r="0.8" fill="#3f7a12" />
    </svg>
  {:else if osKey === 'mint'}
    <!-- Linux Mint -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#68B723" />
      <path d="M7 16V9h2.1l2.3 3.3L13.7 9h2.1v7h-1.9v-4.2l-2.2 3.2-2.2-3.2V16H7z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'kali'}
    <!-- Kali Linux "K" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0a0a0a" />
      <path d="M8.5 7v10M8.5 12l5-5M8.5 12l5 5" fill="none" stroke="#557C94" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  {:else if osKey === 'popos'}
    <!-- Pop!_OS exclamation emblem -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#48B9C7" />
      <rect x="10.5" y="6" width="3" height="7" rx="1.5" fill="#ffffff" />
      <circle cx="12" cy="16.5" r="1.5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'gentoo'}
    <!-- Gentoo "g" swirl -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#54487A" />
      <path d="M13.2 8a4 4 0 1 0-3.6 5.7" fill="none" stroke="#ffffff" stroke-width="2.3" stroke-linecap="round" />
      <path d="M9.6 13.7c0 1.8 1.4 3.2 3.2 3.2" fill="none" stroke="#ffffff" stroke-width="2.3" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'nixos'}
    <!-- NixOS snowflake -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1E2A45" />
      <g stroke="#5277C3" stroke-width="2.1" stroke-linecap="round">
        <line x1="12" y1="5.3" x2="12" y2="18.7" />
        <line x1="6.2" y1="8.7" x2="17.8" y2="15.3" />
        <line x1="17.8" y1="8.7" x2="6.2" y2="15.3" />
      </g>
      <circle cx="12" cy="12" r="1.5" fill="#8fc0f0" />
    </svg>
  {:else if osKey === 'raspberry'}
    <!-- Raspberry Pi -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1b1b1b" />
      <circle cx="9" cy="10" r="1.4" fill="#C51A4A" />
      <circle cx="12" cy="9" r="1.4" fill="#C51A4A" />
      <circle cx="15" cy="10" r="1.4" fill="#C51A4A" />
      <circle cx="8.5" cy="13" r="1.4" fill="#C51A4A" />
      <circle cx="11.5" cy="12.6" r="1.4" fill="#C51A4A" />
      <circle cx="14.5" cy="13" r="1.4" fill="#C51A4A" />
      <circle cx="11" cy="15.6" r="1.4" fill="#C51A4A" />
      <circle cx="14" cy="16" r="1.4" fill="#C51A4A" />
      <path d="M9.5 7.8c1-1.3 2.8-1.7 4.2-.9" fill="none" stroke="#75A928" stroke-width="1.5" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'amazon'}
    <!-- Amazon Linux (AWS smile) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#232F3E" />
      <path d="M6.5 14.5c3.6 2.3 7.4 2.3 11 0" fill="none" stroke="#FF9900" stroke-width="2" stroke-linecap="round" />
      <path d="M15.6 13.2l1.9 1-.4 2.1" fill="none" stroke="#FF9900" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  {:else if osKey === 'oracle'}
    <!-- Oracle Linux -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#F80000" />
      <circle cx="12" cy="12" r="5" fill="none" stroke="#ffffff" stroke-width="2.4" />
    </svg>
  {:else if osKey === 'void'}
    <!-- Void Linux -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#16211c" />
      <circle cx="12" cy="12" r="7" fill="none" stroke="#478061" stroke-width="2.2" />
      <circle cx="12" cy="12" r="2.6" fill="#478061" />
    </svg>
  {:else if osKey === 'endeavour'}
    <!-- EndeavourOS rocket -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#7F3FBF" />
      <path d="M12 5.5c2 2.6 3 5.4 3 8.5H9c0-3.1 1-5.9 3-8.5z" fill="#ffffff" />
      <path d="M9 14.5l-2 3M15 14.5l2 3M10.5 17h3" stroke="#ffffff" stroke-width="1.4" stroke-linecap="round" />
      <circle cx="12" cy="10.2" r="1.1" fill="#7F3FBF" />
    </svg>
  {:else if osKey === 'elementary'}
    <!-- elementary OS "e" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#4A90D9" />
      <path d="M8 12.4a4.1 4.1 0 1 0 1.4-3.1" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" />
      <path d="M7.9 12.2h6.4" stroke="#ffffff" stroke-width="2" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'zorin'}
    <!-- Zorin OS "Z" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0CC1E8" />
      <path d="M8.3 8h7.4l-6 8H16" fill="none" stroke="#ffffff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'slackware'}
    <!-- Slackware "S" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1a3f6b" />
      <path d="M15.3 8.4a5 5 0 0 0-3-.9c-1.8 0-3.1.9-3.1 2.1 0 1.1 1 1.6 2.7 2l1 .2c2 .5 3.1 1.4 3.1 2.9 0 1.9-1.7 3.2-4.2 3.2-1.5 0-2.9-.4-4-1.2" fill="none" stroke="#ffffff" stroke-width="1.7" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'mageia'}
    <!-- Mageia star -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1D3E6E" />
      <path d="M12 4.2l1.8 5.5 5.6-1.6-3.9 4.3 3.9 4.3-5.6-1.6-1.8 5.5-1.8-5.5-5.6 1.6 3.9-4.3-3.9-4.3 5.6 1.6z" fill="#59A5D8" />
    </svg>
  {:else if osKey === 'solus'}
    <!-- Solus spiral -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#4a4de7" />
      <path d="M9.2 8.2a5 5 0 1 0 5 5" fill="none" stroke="#ffffff" stroke-width="2" stroke-linecap="round" />
      <circle cx="14" cy="8" r="1.3" fill="#ffffff" />
    </svg>
  {:else if osKey === 'tails'}
    <!-- Tails -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#3B1F47" />
      <path d="M7 16c1-4 3-7 8-8" fill="none" stroke="#ffffff" stroke-width="1.8" stroke-linecap="round" />
      <circle cx="16" cy="7.3" r="1.1" fill="#ffffff" />
    </svg>
  {:else if osKey === 'deepin'}
    <!-- Deepin droplet -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#007AFF" />
      <path d="M12 5.5c2.8 3 4.5 5.4 4.5 7.8a4.5 4.5 0 1 1-9 0c0-2.4 1.7-4.8 4.5-7.8z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'clear'}
    <!-- Clear Linux "C" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0071C5" />
      <path d="M15.2 8a5 5 0 1 0 0 8" fill="none" stroke="#ffffff" stroke-width="2.2" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'garuda'}
    <!-- Garuda Linux wing -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#F75229" />
      <path d="M5 14c3-3.5 6-5.5 9-6.5-1 3-2.5 6-5 8.5 2-.3 4.5-1.5 6.5-3.5-2 4-6 6-10.5 4.5z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'steam'}
    <!-- SteamOS -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#171A21" />
      <circle cx="9.3" cy="14.7" r="2.5" fill="none" stroke="#66c0f4" stroke-width="1.5" />
      <circle cx="14.8" cy="9" r="3.3" fill="none" stroke="#66c0f4" stroke-width="1.5" />
      <circle cx="14.8" cy="9" r="1" fill="#66c0f4" />
      <path d="M11.1 12.9l2.2-2.2" stroke="#66c0f4" stroke-width="1.5" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'flatcar' || osKey === 'coreos'}
    <!-- Flatcar / CoreOS -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0c0c0c" />
      <path d="M12 5.2l5.8 3.4v6.8L12 18.8l-5.8-3.4V8.6L12 5.2z" fill="none" stroke="#E6522C" stroke-width="1.7" stroke-linejoin="round" />
      <circle cx="12" cy="12" r="2.1" fill="#E6522C" />
    </svg>
  {:else if osKey === 'devuan'}
    <!-- Devuan "D" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#39424a" />
      <path d="M9 6.8v10.4h2.8a5.2 5.2 0 0 0 0-10.4H9z" fill="none" stroke="#ffffff" stroke-width="1.7" stroke-linejoin="round" />
    </svg>
  {:else if osKey === 'parrot'}
    <!-- Parrot OS -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#00D0FF" />
      <path d="M8 15c0-4 2.5-7 6-7 2 0 3.5 1.2 3.5 2.8 0 1.7-1.6 2.5-3.3 2.1L18 15l-4.2-.4c.4 1.2-.2 2.4-1.5 2.7-1.6.4-3.3-.3-4.3-2.3z" fill="#0b1728" />
    </svg>
  {:else if osKey === 'mx'}
    <!-- MX Linux -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1c1c1c" />
      <text x="12" y="15.6" text-anchor="middle" font-family="system-ui, sans-serif" font-weight="700" font-size="8.5" fill="#ffffff">MX</text>
    </svg>
  {:else if osKey === 'kubuntu'}
    <!-- Kubuntu (KDE blue) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0079C1" />
      <circle cx="12" cy="12" r="6.3" fill="none" stroke="#ffffff" stroke-width="1.5" />
      <text x="12" y="15.4" text-anchor="middle" font-family="system-ui, sans-serif" font-weight="700" font-size="7.5" fill="#ffffff">K</text>
    </svg>
  {:else if osKey === 'xubuntu'}
    <!-- Xubuntu (Xfce teal) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1B6D85" />
      <circle cx="12" cy="12" r="6.3" fill="none" stroke="#ffffff" stroke-width="1.5" />
      <text x="12" y="15.4" text-anchor="middle" font-family="system-ui, sans-serif" font-weight="700" font-size="7.5" fill="#ffffff">X</text>
    </svg>
  {:else if osKey === 'lubuntu'}
    <!-- Lubuntu (LXQt indigo) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0068C8" />
      <circle cx="12" cy="12" r="6.3" fill="none" stroke="#ffffff" stroke-width="1.5" />
      <text x="12" y="15.4" text-anchor="middle" font-family="system-ui, sans-serif" font-weight="700" font-size="7.5" fill="#ffffff">L</text>
    </svg>
  {:else if osKey === 'freebsd'}
    <!-- FreeBSD "Beastie" head -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#AB2B28" />
      <path d="M8.5 8.2L7 6M15.5 8.2L17 6" stroke="#ffffff" stroke-width="1.6" stroke-linecap="round" />
      <circle cx="12" cy="12.5" r="5" fill="#ffffff" />
      <circle cx="10.1" cy="11.5" r="0.9" fill="#AB2B28" />
      <circle cx="13.9" cy="11.5" r="0.9" fill="#AB2B28" />
      <path d="M9.7 14.7c1.6 1.2 3.4 1.2 5 0" stroke="#AB2B28" stroke-width="1" fill="none" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'openbsd'}
    <!-- OpenBSD "Puffy" -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#F2BE22" />
      <path d="M6 9l-1.6-1M18 9l1.6-1M6.4 14.5l-1.9.6M17.6 14.5l1.9.6" stroke="#1f2937" stroke-width="1" stroke-linecap="round" />
      <circle cx="8.7" cy="11" r="1.3" fill="#1f2937" />
      <circle cx="15.3" cy="11" r="1.3" fill="#1f2937" />
      <path d="M8.5 15c1.7 1.4 5.3 1.4 7 0" fill="none" stroke="#1f2937" stroke-width="1.6" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'netbsd'}
    <!-- NetBSD flag -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1b1b1b" />
      <path d="M8.5 6v12M8.5 6h6.5L12.5 9l2.5 3H8.5" fill="none" stroke="#FF6600" stroke-width="1.7" stroke-linejoin="round" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'mikrotik'}
    <!-- MikroTik / RouterOS -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#2A5298" />
      <rect x="5.5" y="9" width="13" height="6" rx="1.5" fill="#ffffff" />
      <circle cx="8.3" cy="12" r="1.1" fill="#2A5298" />
      <circle cx="12" cy="12" r="1.1" fill="#2A5298" />
      <circle cx="15.7" cy="12" r="1.1" fill="#2A5298" />
    </svg>
  {:else if osKey === 'cisco'}
    <!-- Cisco bridge bars -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#049FD9" />
      <path d="M6 14v3M8.7 11v6M11.4 8v9M14 11v6M16.7 8v9M19.4 11v6" stroke="#ffffff" stroke-width="1.5" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'windows'}
    <!-- Windows -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0078D4" />
      <rect x="6" y="6" width="5" height="5" fill="#ffffff" />
      <rect x="13" y="6" width="5" height="5" fill="#ffffff" />
      <rect x="6" y="13" width="5" height="5" fill="#ffffff" />
      <rect x="13" y="13" width="5" height="5" fill="#ffffff" />
    </svg>
  {:else if osKey === 'macos'}
    <!-- Apple (macOS) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#1d1d1f" />
      <path d="M14.7 8.1c-.6-.7-1.4-1.1-2.3-1.1-.2-1 .3-2 1-2.6.2.9-.2 1.8-.9 2.4.9-.1 1.7.3 2.2 1.3z" fill="#ffffff" />
      <path d="M15.9 9.6c-1.1-.7-2.4-.6-3.4.2-.9-.6-2-.7-3-.3-1.6.6-2.5 2.3-2.3 4 .2 1.8 1 3.6 2 5 .6.8 1.3 1.6 2.2 1.6.7 0 1-.4 1.8-.4s1.1.4 1.8.4c1 0 1.6-.8 2.2-1.6.5-.7.8-1.4 1.1-2.2-1.6-.6-2.4-2.9-1-4.1.4-.4.8-.6 1.2-.7-.3-.8-.8-1.4-1.6-1.9z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'ios'}
    <!-- Apple (iOS) -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0A84FF" />
      <path d="M14.7 8.1c-.6-.7-1.4-1.1-2.3-1.1-.2-1 .3-2 1-2.6.2.9-.2 1.8-.9 2.4.9-.1 1.7.3 2.2 1.3z" fill="#ffffff" />
      <path d="M15.9 9.6c-1.1-.7-2.4-.6-3.4.2-.9-.6-2-.7-3-.3-1.6.6-2.5 2.3-2.3 4 .2 1.8 1 3.6 2 5 .6.8 1.3 1.6 2.2 1.6.7 0 1-.4 1.8-.4s1.1.4 1.8.4c1 0 1.6-.8 2.2-1.6.5-.7.8-1.4 1.1-2.2-1.6-.6-2.4-2.9-1-4.1.4-.4.8-.6 1.2-.7-.3-.8-.8-1.4-1.6-1.9z" fill="#ffffff" />
    </svg>
  {:else if osKey === 'android'}
    <!-- Android bugdroid -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0a0a0a" />
      <path d="M8.3 6.6L7.2 5.2M15.7 6.6l1.1-1.4" stroke="#3DDC84" stroke-width="1.1" stroke-linecap="round" />
      <path d="M8 10.2a4 4 0 0 1 8 0z" fill="#3DDC84" />
      <circle cx="10" cy="8.7" r="0.5" fill="#0a0a0a" />
      <circle cx="14" cy="8.7" r="0.5" fill="#0a0a0a" />
      <path d="M8.3 10.6v4a.85.85 0 0 0 1.7 0v-4M14 10.6v4a.85.85 0 0 0 1.7 0v-4" stroke="#3DDC84" stroke-width="1.5" fill="none" stroke-linecap="round" />
    </svg>
  {:else if osKey === 'server'}
    <!-- Generic server rack -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#0f172a" />
      <rect x="5.7" y="6" width="12.6" height="4.5" rx="1" fill="none" stroke="#38bdf8" stroke-width="1.5" />
      <rect x="5.7" y="13.5" width="12.6" height="4.5" rx="1" fill="none" stroke="#38bdf8" stroke-width="1.5" />
      <circle cx="8.3" cy="8.25" r="0.8" fill="#38bdf8" />
      <circle cx="8.3" cy="15.75" r="0.8" fill="#38bdf8" />
    </svg>
  {:else}
    <!-- Generic Linux (Tux) fallback -->
    <svg viewBox="0 0 24 24" width={size} height={size}>
      <circle cx="12" cy="12" r="10" fill="#EDEDED" />
      <path d="M12 4.5c-2.1 0-3.6 1.9-3.4 4.1.1.9.4 1.7.9 2.4-1.7 1.5-2.9 4.1-2.9 6.6 0 1 .8 1.4 1.8 1.1.4-.1.7-.4.8-.8h5.6c.1.4.4.7.8.8 1 .3 1.8-.1 1.8-1.1 0-2.5-1.2-5.1-2.9-6.6.5-.7.8-1.5.9-2.4.2-2.2-1.3-4.1-3.4-4.1z" fill="#0b0b0b" />
      <ellipse cx="12" cy="15.5" rx="2.4" ry="3.8" fill="#ffffff" />
      <ellipse cx="10.7" cy="9.2" rx="0.9" ry="1.1" fill="#ffffff" />
      <ellipse cx="13.3" cy="9.2" rx="0.9" ry="1.1" fill="#ffffff" />
      <path d="M11.2 10.8l.8.6.8-.6" fill="none" stroke="#F5A623" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
      <path d="M9.6 19.6l.9-1.5M14.4 19.6l-.9-1.5" stroke="#F5A623" stroke-width="1.6" stroke-linecap="round" />
    </svg>
  {/if}
</span>
