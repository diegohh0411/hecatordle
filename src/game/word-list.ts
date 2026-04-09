const easyWords = [
  "about","angel","apple","awake","beach","black","bread","brain","brown","candy",
  "chair","clean","cloud","dream","early","faith","fresh","funny","grace","grape",
  "green","happy","heart","honey","house","large","later","light","lucky","magic",
  "month","music","night","novel","ocean","older","other","party","peace","plant",
  "power","quick","river","salty","small","smile","sound","spice","stone","story",
  "storm","sugar","super","sweet","table","their","there","today","trust","voice",
  "water","which","white","world","years","young","zesty","pride","trace","style"
] as const;

const normalWords = [
  "adobe","align","argue","brisk","clerk","craft","crane","delta","diner","ember",
  "fable","flint","gauge","glint","haste","hover","irony","jolly","karma","knack",
  "lemon","linen","mango","medal","merit","noble","occur","olive","orbit","pearl",
  "phase","plaid","quilt","radar","ratio","relay","renew","rhyme","ridge","rumor",
  "rural","scale","scope","shard","sheet","shift","shiny","siege","skate","skill",
  "slate","spare","spear","spoke","stack","stare","steam","stint","strap","swirl",
  "tango","tempo","trial","ultra","vivid","woven","waltz","caper","apron","creek"
] as const;

const hardWords = [
  "aback","axiom","azure","banal","bokeh","cacao","cairn","crypt","cynic","dizzy",
  "dowry","eclat","ennui","feint","fjord","fuzzy","glyph","gnash","guile","ivory",
  "jaunt","jazzy","khaki","llama","mauve","nadir","nymph","ovoid","pique","quark",
  "quipu","radii","slyly","taffy","umbra","vixen","wryly","xylem","yacht","zonal"
] as const;

export const EASY_WORDS = [...easyWords];
export const NORMAL_WORDS = [...normalWords];
export const HARD_WORDS = [...hardWords];

export const WORD_LIST = Array.from(
  new Set([...EASY_WORDS, ...NORMAL_WORDS, ...HARD_WORDS]),
);

export const WORD_SET: ReadonlySet<string> = new Set(WORD_LIST);
