export interface Sponsor {
  name: string;
  avatar: string;
  tier: SponsorTierId;
}

export type SponsorTierId = 'platinum' | 'gold' | 'silver';

export const SPONSOR_TIERS = [
  { id: 'platinum', label: 'Platinum Sponsor' },
  { id: 'gold', label: 'Gold Sponsor' },
  { id: 'silver', label: 'Silver Sponsor' }
];

export const SPONSORS: Sponsor[] = [
  {
    name: 'Fathforce Ecosystem',
    avatar: 'https://avatars.githubusercontent.com/u/1000000?v=4',
    tier: 'platinum'
  }
];
