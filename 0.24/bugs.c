// Bugs - a genetic programming experiment
// Inspired by the 1992 Scientific American article that I have not yet read
// (c) 2003 Mike Inman - all rights reserved

// Revision history
//
// 0.09 28Nov2003
//   Initiate Revision History
//   Add position and family history to bug dump
//   Move tweakgene into a function, add possibility of multiple adjustments in a tweak
//   Also add possibility of tweakgene call on a newly added gene
//   Adjusted bug_one to reproduce more easily, increased FOODSTART and radically decreased FOODSPREAD
//   Modified GENECOST to work on ngenes cubed divided by GENEKNEE squared
//   Added expression filter to only evaluate one (randomly selected) chromosome per function
//   Changed grand prize to slasher prize
//
// 0.10 30Nov2003
//   Drop COSTMATE from 70 to 15
//   Bump COSTMOVE from 37 to 38
//   Add LEFTBAR status graph
//   Add gene distribution within chromosomes to bug dump (text) report
//   Revised leangenes prize to function with short lived bugs
//
// 0.11 04Dec2003
//   Add a series of dynamic challenges:
//     At year 0.5, turn off the food leak into cells containing bugs (eliminate sedentary bugs)
//     Add dynamic geneknee2 variable that slowly increases cost of genes - objective: building a better bug_one for later versions
//     Force mating, not for bug one, but once starting at Q1 and once per brood starting at Q3
//   Change annual reports to quarterly
//   Took additive age out of slasher formula
//
// 0.12 07Dec2003
//   Add forcemate 3 and 4 levels for additional cost to dividing without mating (actually forces intelligent divide decisions)
//   Rearrange forcemate escalation to take place more quickly and up through 4 now
//   Separate seasonal foodgrowth into its own function (easing into the idea of "three continents")
//   Revise mechanics of division so that offspring are born to the rear (reduces mating with children)
//   Accelerated "genesqueeze" to once per 256 turns (from 512)
//   Increased FOODSPREAD from 1 to 10, note that the leak is turned off to the bugs at 4096
//   Revised bugone move to be less involved
//   Add dynamic costmate to slowly increase the cost of mating
//
// 0.13 12Dec2003
//   Add forcemate 5, which actually requires bug to be a certain age before dividing
//   Incrementally ramp up the age of division to encourage longevity - with the simultaneous objective of lowering starvation rates
//   Comment out dynamic geneknee code - removes encouragement for simple genomes
//   Increase costmate by geometric progression instead of incrementing every 1024 turns
//   Add selfage sense (might be useful in holding off premature division decisions)   
//   Copied evolved gene sets from year 11.08 of Bugs 0.12 for use in bug_one
//   Evolved genes were not spontaneously dividing in one bug environment - restored divide and turn decision to original bug_one genes
//   Change forcemate to a binary flag decision variable instead of levels - modified progression to later turn off mating requirements
//   Added population driven control of agediv - if population dips too low, drop agediv to make it easier
//   Adjusted COSTMOVE 38-35 COSTTURN 10-15 COSTSLEEP 1-10
//
// 0.14 16Dec2003
//   Widen blackout food growth around bugs to a distance of up to 3 cells instead of one, at one cell bugs would be born, 
//     eat, turn, then give birth into an adjacent cell where food has grown in.  Not a terribly interesting life.
//   Added rot[] array defining food decay rates around bugs
//   Tweaked random seed - noted a high degree of dependance upon the random seed for outcome...
//   Fixed MAJOR bug with move action - reduced rot distance to 1 cell - increased world size to work with new ~3% density
//
// 0.15 20Dec2003
//   Change population density limit from 2% to 4%
//   Change food leak back to single cell exclusion (from 7 cell)
//   Added three zone cosine terrain
//   Added ethnicity
//   Added color-trace ethnicity plot
//   Switched seasonal sweep to right-to-left from bottom-to-top
//   Revamped cost structure to increase cost of movement
//   Added colorful gene and parent tracking
//
// 0.16 22Dec2003
//   Tweak population limit to 24000 from 16384
//   Tweak ETHNIC_DUR to 32 from 40
//   Change _ethnicity implementation to chars from shorts
//   Put costmate on a progressive escalation
//   Start food growth regulation based on age of division (thus, indirectly based on population)
//
// 0.17 05Jan2004
//   Drop mating requirement
//   Increase ETHNIC_DUR to 120
//
// 0.18 07Jan2004
//   Restore mating requirement
//   Implement dynamic target population - reducing over time
//   Revise dynamic food supply to smaller changes more often (net higher mobility) and target agediv to 30-300 from 10-100
//   Increase geneknee from 36 to 48
//
//   COMET STRIKE IN YEAR 75!  -  believed to be tied to an overflow condition in the growing_season function, a radical
//     reduction in food supplies leads to mass starvation near the end of Year 75, ultimately leading to extinction in 
//     the 0.18 batch of bugs.  Decision made to leave this condition in place, 2.5 million turns is enough.
//
// 0.19 15Feb2004
//   Implement on/off mating requirement
//   Increase report frequency
//
// 0.20 28Feb2004
//   Decrease targetpop 50% to 12000
//   Decrease genecost 67% to 128
//   Increase geneknee 2x to 96
//
// 0.21 02Mar2004
//   Increase agressiveness of AgeDiv such that AgeDiv = age of oldest bug when population exceeds POP_HARDLIMIT (25000)
//
// 0.22 02July2004
//   Rerun version 21
//
// Revisit on new machine 12Mar2005
//   Change include <fastmath.h> to <math.h>
//
// 0.23 13Mar2005
//   Increase targetpop     from 12000 to 120000
//   Increase POP_HARDLIMIT from 24000 to 130000
//
// 0.24 15Mar2005
//   Add a FOODCAP*10 hardlimit on food levels
//   Boost foodhump to 10.0 (from 1.4) when population first passes 1000, (for increased population density, 1.4 limits around 14,000)
//   Add MASSCAP medium-hard limit on bug girth
//  Change leftside genes plot to go from min to maxgenes instead of 0 to maxgenes
//
// Future ideas:
//   Aging representation in bottom graph - break white population marker into 4 zones, <10, <100, <1000, >1000 with shades for each
//   Salmon sense - record birthplace and give sense of distance and direction relative to it
//   Navigational sense - ability to know longitude and latitude and compass direction
//   Switch to "wide screen" 16:9 format, maybe 960x540?
//   Use "sign corrected" square root to flatten the food profile peaks and widen the valleys
//   Slowly increase cost of reproduction 


#include <gd.h>
#include <math.h>
#include <stdio.h>

// Note on the linked list structures
// Lists are maintained by a top pointer, either the buglist or chromosomes within the bug
// Lists are double-linked, with *next == NULL on the last item and *prev == NULL on the first item
// the empty list case is signalled by the top pointer == NULL

#define WORLD_X         192
#define WORLD_Y         160
#define LEFTBAR          80
#define RIGHTBAR         80
#define SIDEBAR          LEFTBAR + RIGHTBAR
#define BOTTOMBAR        80
#define SEASONLENGTH  32768
#define FOODCAP     1024000   /* cap out at x food per cell - food values recorded * 1024              */
#define FOODGROW       1044   /* food multiplies by x per turn (day)                                   */
#define FOODSHADOW      973   /* food decays when bug is sitting on cell                               */
#define FOODSPREAD       10   /* food spreads into poorer adjacent cells at x% per turn                */
#define FOODSTART    128000
#define FOODDECAY       115   /* Rate at which overages decay                                          */
#define COSTSLEEP        12
#define COSTEAT          48
#define COSTTURN         16
#define COSTMOVE         96
#define COSTFIGHT        36  /* additional cost on top of moving                                         */
#define COSTMATE         12  
#define COSTDIVIDE    25600  /* cost per resulting creature (ex: divide into 3, child = parent / 3 - 25) */
#define NOMMASS        1024  /* nominal mass, costs are prorated according to COST*mass/NOMMASS          */
#define GENECOST        128
#define GENEKNEE         96  /* non-linearity inflection point, beyond the knee, genecost increases steeply */
#define EATLIMIT        205      /* allow eating 20% of body mass per turn                                   */
#define DIETHIN      102400    /* limit at which bug starves and becomes bugfood                           */
#define MASSCAP   10240000  /* above 10000, start the masscap tax */
#define ACTSLEEP          0  /* action index numbers                                                     */
#define ACTEAT            1  /* action with the highest weight is the action taken                       */
#define ACTTURNCW         2
#define ACTTURNCCW        3
#define ACTMOVE           4  /* if no bug in destination cell, simply move, otherwise fight bug in dest. */
#define ACTMATE           5  /* if no bug in destination cell, waste COSTMATE energy and do nothing      */
#define ACTDIVIDE         6
#define RESPONSEMATE      7  /* if response mate >0, mate is the decision, otherwise, ignore             */  
#define ACTMATED          7  /* also for logging, agressee does not use turn                             */
#define ACTDEFEND         8  /* for history logging, does not consume turn                               */
#define NACT              9  /* number of actions for history computation, response mate is not recorded */
#define NDECISIONS        8  
#define POSHISTORY       32  /* retain position history for 32 turns */
#define ITEMFOOD          0  /* mass of food in cell */
#define ITEMBUG           1  /* mass of bug in cell, 0 if none */
#define ITEMBUGFACE       2  /* translated bug facing, or 0 if no bug */
#define ITEMBUGMATCH      3  /* genetic match of bug 0-1.0, or 0 if no bug */
#define DIR_E             0
#define DIR_NE           -1
#define DIR_NW           -2
#define DIR_SE            1
#define DIR_SW            2
#define DIR_W             3
#define DIR_CW            1
#define DIR_CCW          -1
#define NONE              0
#define NSENSECELLS      12
#define SENSESELF         NSENSECELLS * 4
#define SPAWNWEIGHTNORM   NSENSECELLS * 4 + NACT  /* special purpose senses */
#define STARVEWEIGHTNORM  SPAWNWEIGHTNORM + 1     
#define SELFAGE           STARVEWEIGHTNORM + 1     
#define NSENSES           SELFAGE + 1    /* number of sense data points on which bug decisions are made */
#define GENECONST         1
#define GENESENSE         2
#define GENELIMIT         3
#define GENECOMPARE       4
#define GENEMATCH         5
#define FAMHIST         126
#define LHIST          1024
#define ETHNIC_DUR      120    /* Number of generations it takes to assimilate to the local color */
#define POP_HARDLIMIT 24000
#define POP_TARGET       5000

struct _pos
  { long x;      // 0 to WORLD_X - 1
    long y;      // 0 to WORLD_Y - 1  
  };

struct _ethnicity
  {  long uid;  // Unique id number - like a serial number
     char r;    // ethnicity weights
     char g;
     char b;
     char pad;  // to keep word alignment of data structure...
  };
 
struct _bugstate
  {  struct _pos p;
            long face;   // which way is the bug pointing (0=E 1=SE 2=SW 3=W -1=NE -2=NW)
            long act;    // action which got us here 0-8
            long weight; // at this time (think fixed point, value is *1024)
  };

typedef struct _gene
  {         long  tp;     // Gene type 1=constant, 2=sense, 3=limit function
            long  si;     // index of the sense function to use (unless type==1)
            long  c1,c2;  // Constants, c1 used in constant type, c1 and c2 used in limit function
    struct _gene *next;   // Housekeeping link for adding and deleting genes from list
    struct _gene *prev;   // Housekeeping link
    struct _gene *prod;   // Evaluation link, value = vself * vprod + vsum
    struct _gene *sum;    // Evaluation link, if *prod == null, vprod = 1.0, if *sum == null, vsum = 0.0
  } Gene;

struct _bugact            // One of these gene sets for each action
  { struct _gene *a,*b;   // will build two lists of "genes", this structure is like a diploid "chromosome"
    struct _ethnicity ea,eb;  // Track the origin of the gene's mutation
  };

struct _bugbrain
  { struct _bugact act[NDECISIONS];    // decision weights
 struct _ethnicity family[FAMHIST];
 struct _ethnicity eth;
              long generation;
              long divide;             // number of children in a division 2-7 
             short ngenes;             // for all act.a and act.b lists, used to adjust cost of living
             short expression;         // bitmap telling which chromosomes get used
  };

typedef struct _bugdata
  {             long  birthday;        // calendar day of last division
                long  kills;           // Other life history
                long  defends;
                long  moves;
                long  mate_success;
                long  mate_fails;
                long  mate_repeat;
                long  offspring;
    struct _bugstate  pos[POSHISTORY]; // 0 is current, others historical
    struct _bugbrain  brain;           // decisionmaker data
    struct _bugbrain  matebrain;       // decisionmaker of last mate, used in sexual procreation, copy of self at birth
    struct  _bugdata *next;            // linked list forward pointer
    struct  _bugdata *prev;            // linked list backward pointer
  } Bugdata;

struct _buglist
  {            long  n_bugs;
    struct _bugdata *first; 
    struct _bugdata *endlist; 
  };

struct _worlddata  // data for one cell of the world
  { struct _bugdata *bug;   // quickfind for the bug in this cell
               long  food;  // might add interesting weather, terrain and other things... later
               long  nearest;  // distance to the nearest bug
  };

struct _historydata
  { long n_bugs;
    long movement;
    long collisions;
    long starvations;
    long births;
    long avgweight;
    long avgfood;
    long avggenes;
  };

//
// global
//
  struct _worlddata  world[WORLD_X][WORLD_Y];
  struct   _buglist  buglist;
    struct _bugdata *nextglobalbug;
struct _historydata  hist[LHIST];             // historical statistics by turn for graphs
               long  sense[NSENSES];          // shared array, used by all bugs one at a time to make decisions
               long  today;                   // day counter
               long  idcounter;               // unique bug identifier
        double long  totalfood;               // status variable
        double long  totalbug;                // status variable
        double long  genecount;               // status variable
               long  db = 0;                  // debug flag
               long  leak = -1;               // Food leak, starts in on state, higher values makes leak stay farther from bug
               long  geneknee2 = GENEKNEE * GENEKNEE;  // For dynamic genecost adjustment
               long  forcemate = 0;
               long  costmate = COSTMATE;
               long  agediv = 0;              // must be at least agediv to successfully divide - stiff penalty otherwise (when forcemate >=5)
               long  rot[4];
               long  safety = 1;
               long  target_pop = POP_TARGET;
              float  foodhump = 1.4;

// functions

//
// Hex grid movement translators - 0,0 is upper left corner of map
//
void east( struct _pos *p )
{ if ( p->x < WORLD_X - 1 )
    p->x++;
   else
    p->x = 0;
}

void southeast( struct _pos *p )
{ if ( p->y % 2 == 0 )
    { if ( p->x < WORLD_X - 1 )
        p->x++;
       else
        p->x = 0;
    }
    
  if ( p->y < WORLD_Y - 1 )
    p->y++;
   else
    p->y = 0;
}

void northeast( struct _pos *p )
{ if ( p->y % 2 == 0 )
    { if ( p->x < WORLD_X - 1 )
        p->x++;
       else
        p->x = 0;
    }

  if ( p->y > 0 )
    p->y--;
   else
    p->y = WORLD_Y - 1;
}

void west( struct _pos *p )
{ if ( p->x > 0 )
    p->x--;
   else
    p->x = WORLD_X - 1;
}

void southwest( struct _pos *p )
{ if ( p->y % 2 != 0 )
    { if ( p->x > 0 )
        p->x--;
       else
        p->x = WORLD_X - 1;
    }

  if ( p->y < WORLD_Y - 1 )
    p->y++;
   else
    p->y = 0;
}

void northwest( struct _pos *p )
{ if ( p->y % 2 != 0 )
    { if ( p->x > 0 )
        p->x--;
       else
        p->x = WORLD_X - 1;
    }

  if ( p->y > 0 )
    p->y--;
   else
    p->y = WORLD_Y - 1;
}

void hexmove( struct _pos *p , long dir )
{ 
  // Compensate for rollovers
  while ( dir < DIR_SW ) 
    dir += 6;
  while ( dir > DIR_W )
    dir -= 6;

  switch ( dir )
    { case DIR_NW: northwest( p ); break;
      case DIR_NE: northeast( p ); break;
      case DIR_E:       east( p ); break;
      case DIR_SE: southeast( p ); break;
      case DIR_SW: southwest( p ); break;
      case DIR_W:       west( p ); break;
    }
}

long myabs( long v )
{ if ( v>=0 ) return v;
  return -v;
}

long limitedrandom( long limit )
{ static long seed = 54321;

  seed = myabs( ((seed + 12355) * 16807) ) % 0x3FFFFFFF;

  return (((unsigned long)seed >> 8) % limit);
}
        

void init_world( void )
{ long x,y;
  for ( x = 0 ; x < WORLD_X ; x++ )
    for ( y = 0 ; y < WORLD_Y ; y++ )
      { world[x][y].food = FOODSTART;
        world[x][y].bug  = NULL;
      }
  today           = 0;
  idcounter       = 0;
  buglist.n_bugs  = 0;
  buglist.first   = NULL;
  buglist.endlist = NULL;

  for ( x = 0 ; x < LHIST ; x++ )
    { hist[x].n_bugs      =
      hist[x].movement    =
      hist[x].collisions  =
      hist[x].starvations =
      hist[x].births      =
      hist[x].avgweight   =
      hist[x].avgfood     =
      hist[x].avggenes    = 0;
    }
}

//
// Determine the seasonal (latitude) growth factor
// Including cosine terrain factor
//
long growing_season( long x, long y )
{  long sax;
  float fgf;

  sax = (x + ( today * WORLD_X ) / SEASONLENGTH) % WORLD_X;   // Seasonally adjusted X

  fgf = 0.1 + foodhump * sin( (3.14159 * ((float)sax)) / ((float)WORLD_X) ) * ( 0.51 - cos( 3.14159 * 6.0 * ((float)y) / ((float)WORLD_Y) ) * 0.5 );
  return (long)(((float)FOODGROW-1024) * fgf) + 1024;
}


void update_nearest( void )
{ long x,y,i,j;
  struct _pos p;

  // set zeroes in all cells with bugs, -1 in others
  for ( x = 0 ; x < WORLD_X ; x++ )
    for ( y = 0 ; y < WORLD_Y ; y++ )
      { if ( world[x][y].bug == NULL )
          world[x][y].nearest = -1;
         else 
          world[x][y].nearest = 0;
      }

  // establish distance to nearest bug - up to 1 bug away for now
//  for ( j = 0 ; j < 3 ; j++ )
//  j = 0;
//    for ( x = 0 ; x < WORLD_X ; x++ )
//      for ( y = 0 ; y < WORLD_Y ; y++ )
//        { if ( world[x][y].nearest == j )
//            for ( i = -2 ; i <= 3 ; i++ )
//              { p.x = x;
//                p.y = y; 
//                hexmove( &p, i );
//                if ( world[p.x][p.y].nearest == -1 )  // Only update cells not yet marked (less or equal)
//                  world[p.x][p.y].nearest = j + 1;
//              }
//        }                
}
  

void grow_food( void )
{ long x,y,i,t;
  struct _pos p;
    long fgl;
   
  update_nearest();

  totalfood = 0;
  totalbug  = 0;
  genecount = 0;

  for ( y = 0 ; y < WORLD_Y ; y++ )
    { 

    for ( x = 0 ; x < WORLD_X ; x++ )
      { 
        fgl = growing_season( x, y );

        if (( world[x][y].nearest == -1 ) || ( leak < world[x][y].nearest ))  // Don't grow the grass when a bug is on it (discourages sedentary bugs)
          world[x][y].food = (world[x][y].food * fgl) / 1024;
         else
          world[x][y].food = (world[x][y].food * rot[world[x][y].nearest] ) / 1024;  // Infact, decay when bug is squatting (even nearby)

        if ( world[x][y].food > FOODCAP )
          world[x][y].food -= ((world[x][y].food - FOODCAP) * FOODDECAY) / 1024;  // Decay 10% of overage per turn - this means foodcap is not a hard limit
	  
        if ( world[x][y].food > FOODCAP * 10 )
          world[x][y].food = FOODCAP * 10;  // This is a hard limit
	  
        totalfood += world[x][y].food / 1024; // more or less

        if ( world[x][y].bug != NULL ) 
          { totalbug  += world[x][y].bug->pos[0].weight; // for reporting
            genecount += world[x][y].bug->brain.ngenes;
          }

        // spread to nearby cells that have less than 50% of this one
        for ( i = -2 ; i <= 3 ; i++ )
          { p.x = x;
            p.y = y;
            hexmove( &p, i );
            if ( world[p.x][p.y].food < world[x][y].food / 16 )
              { if (( world[p.x][p.y].nearest == -1 ) || ( leak < world[p.x][p.y].nearest ))
                  { t = ( world[x][y].food * FOODSPREAD ) / 1024;
                    world[x][y].food -= t;
                    world[p.x][p.y].food += t;
                  }
              }
          }     
           
    } }
}

//
// Search a range for matches, return the number found
//
long range_match( struct _bugbrain *b1, struct _bugbrain *b2, long s1, long e1, long s2, long e2 )
{ long c2,m;

  m = 0;
  while( s1 <= e1 )
    { c2 = s2;
      while ( c2 <= e2 )
        { if ( b1->family[s1].uid == b2->family[c2].uid )
            m++;
          c2++;
        }
      s1++;
    }

  return m;
}


//
// return a number indicating relationship - brother sister = 1024
//
long family_match( struct _bugbrain *b1, struct _bugbrain *b2, long level )
{ long r;
  
  if ( level == 0 ) return 1024;  // self = total match

  r = 0;
  // Mother father matches add 256 each, grandparents 64, etc.
  r += range_match( b1, b2,  0,  1,  0,  1 ) * 256; if ( r == 512 ) return 1024; // same 2 parents, total match
  if ( level == 3 ) return r;
  r += range_match( b1, b2,  2,  5,  2,  5 ) * 64;    // 256 poss
  if ( level == 2 ) return r;
  r += range_match( b1, b2,  6, 13,  6, 13 ) * 16;    // 128 poss
  r += range_match( b1, b2, 14, 29, 14, 29 ) * 4;     //  64 poss
  r += range_match( b1, b2, 30, 62, 30, 62 );         //  32 poss
  // best information on closest bug (potential mate)

  return r;

}
//
// For the passed bug, arrange processed sense data into sense[] global
//
void gather_senses( struct _bugdata *bug )
{ struct _pos cp;
         long i,j,f,level;


  // cells  ahead 1,2,5
  // cells to  cw 4,8,9,11
  // cells to ccw 3,6,7,10

  for ( i = 0 ; i < NSENSECELLS ; i++ )
    { // navigate to the cells
      cp = bug->pos[0].p;
      switch ( i )
        { case  0:                                                                                                       level = 0; break;
          case  1: hexmove( &cp, bug->pos[0].face );                                                                     level = 1; break;
          case  2: hexmove( &cp, bug->pos[0].face ); hexmove( &cp, bug->pos[0].face );                                   level = 2; break;
          case  3: hexmove( &cp, bug->pos[0].face + DIR_CCW );                                                           level = 2; break;
          case  4: hexmove( &cp, bug->pos[0].face + DIR_CW  );                                                           level = 2; break;
          case  5: hexmove( &cp, bug->pos[0].face ); hexmove( &cp, bug->pos[0].face ); hexmove( &cp, bug->pos[0].face ); level = 3; break;
          case  6: hexmove( &cp, bug->pos[0].face + DIR_CCW ); hexmove( &cp, bug->pos[0].face + DIR_CCW );               level = 3; break;
          case  7: hexmove( &cp, bug->pos[0].face + DIR_CCW ); hexmove( &cp, bug->pos[0].face );                         level = 3; break;
          case  8: hexmove( &cp, bug->pos[0].face + DIR_CW  ); hexmove( &cp, bug->pos[0].face );                         level = 3; break;
          case  9: hexmove( &cp, bug->pos[0].face + DIR_CW  ); hexmove( &cp, bug->pos[0].face + DIR_CW  );               level = 3; break;
          case 10: hexmove( &cp, bug->pos[0].face + DIR_CCW * 2 );                                                       level = 3; break;
          case 11: hexmove( &cp, bug->pos[0].face + DIR_CW  * 2 );                                                       level = 3; break;
        }

       // Normalize the food mass to the bug mass
       if ( bug->pos[0].weight <= 0 ) 
         bug->pos[0].weight = 1;  // Protect against potential divide by zeroes

       sense[i] = (world[cp.x][cp.y].food * 1024) / bug->pos[0].weight;

       // is there a bug in the cell?
       if ( world[cp.x][cp.y].bug == NULL )
         { // No, set zero values
           sense[i+NSENSECELLS  ] = 0;
           sense[i+NSENSECELLS*2] = 0;
           sense[i+NSENSECELLS*3] = 0;
         }
        else
         { // Normalize the other bug mass to this bug mass
           sense[i+NSENSECELLS] = (world[cp.x][cp.y].bug->pos[0].weight * 1024) / bug->pos[0].weight;
           // Orient the other bug facing relative to this bug facing (0 = facing same direction)
           f = world[cp.x][cp.y].bug->pos[0].face - bug->pos[0].face;
           while ( f < -2 ) f += 6;
           while ( f >  3 ) f -= 6;
           // normalize facing value -2048 -> 3072 to scale similarly to mass and other decision variables
           sense[i+NSENSECELLS*2] = f * 1024;
           // Family history match 0-1024
           sense[i+NSENSECELLS*3] = family_match( &(world[cp.x][cp.y].bug->brain), &(bug->brain), level );
         }
    }   

  // Add the self-awareness variables
  for ( i = 0 ; i < NACT ; i++ )
    { j = 0;
      while ( j < POSHISTORY ) 
        { if ( bug->pos[j].act == i )                              // Time since act has happened
            { sense[i+NSENSECELLS*4] = (j * 1024) / POSHISTORY;    // Normalized so length of history = 1.0
              j = POSHISTORY;
            }
           else
            { if ( j == POSHISTORY-1 ) 
                sense[i+NSENSECELLS*4] = 1024;                     // This act is not in history
            }
          j++;
        }
    }
  // Might also add a sum of number of times an act has happened in history...

  sense[  SPAWNWEIGHTNORM ] = ((( bug->pos[0].weight / bug->brain.divide ) - COSTDIVIDE ) * 1024 ) / DIETHIN;  // an answer of 1024 means division would yield offspring that must eat immediately or die
  sense[ STARVEWEIGHTNORM ] = ( bug->pos[0].weight * 1024 ) / DIETHIN;  // an answer of 1024 means we're dying of starvation
  sense[          SELFAGE ] = today - bug->birthday;
  
  // Might add a travel history... many other things
  // for ( k = 1; k < POSHISTORY ; k++ )
  //   { i = bug->pos[0].p.x - bug->pos[POSHISTORY - 1].p.x; i = i*i;
  //     j = bug->pos[0].p.y - bug->pos[POSHISTORY - 1].p.y; j = j*j;
  //   }

 
}

//
// limit function
// 
long limit_fn( long x, long l1, long l2 )
{ 
  if ( l1 <= l2 )
    { if ( x < l1 ) return 0;
      if ( x > l2 ) return 1024;

       // inbetween, do a linear interpolation
       if ( l1 = l2 ) return 512;
       return (1024 * ( x - l1 )) / (l2 - l1); 
    }

  // l1 is greater than l2, invert the limit structure
  if ( x < l2 ) return 1024;
  if ( x > l1 ) return 0;

  return 1024 - (1024 * ( x - l2 )) / (l1 - l2); 
}  


long evaluate_gene( struct _gene *g )
{ long v;

  if ( g->si < 0 )       { g->si = 0;                        printf("Hosed.\n"); }
  if ( g->si > NSENSES ) { g->si = limitedrandom( NSENSES ); printf("Hosed.\n"); } // this should never happen, but...

  switch ( g->tp )
    { case GENECONST:
        v = g->c1;
        break;
 
      case GENESENSE:
        v = (sense[g->si] * g->c1) / 1024 + g->c2;
        break;

      case GENELIMIT:
      default:
        v = limit_fn( sense[g->si], g->c1, g->c2 );
        break;

      case GENECOMPARE:
        v = (sense[g->si] - sense[(g->c1) % NSENSES]) + g->c2;

      case GENEMATCH:
        v = 1024 - myabs((sense[g->si] - sense[(g->c2) % NSENSES]) * g->c1) / 1024;
        if ( v < 0 ) v = 0;
    }
  
  if ( g->prod != NULL )
    v = (v * evaluate_gene( g->prod )) / 1024;

  if ( g->sum != NULL )
    v += evaluate_gene( g->sum );

  return v;
}

//
// Evaluate the current sense[] to decide on an action
//
long bugdecide( struct _bugbrain *brp )
{ long actv[NDECISIONS];
  long i,j,x,v,maxv;

  maxv = -1048576;
  j = 0;
  x = 1;
  for ( i = 0 ; i <= ACTDIVIDE ; i++ )
    { if ( (brp->expression & x) != 0 )
        v = evaluate_gene( brp->act[i].a ); 
       else 
        v = evaluate_gene( brp->act[i].b );
      x = x * 2;
      if ( v > maxv )
        { maxv = v;
          j = i;
        }
    }
  return j;
}

//
// adjust the mass for normalized cost extraction
// 
void costcalc( long cost, struct _bugdata *bug )
{ long mass;

  mass = myabs( bug->pos[0].weight ) + (GENECOST * bug->brain.ngenes * bug->brain.ngenes * bug->brain.ngenes) / geneknee2;  // For cost calculations each gene is counted as additional mass

  // Special obesity tax 1% per unit over masscap (100 units over doubles cost of everything, 200 triples, etc.)
  if ( mass > MASSCAP )
    cost = cost * (1 + (mass - MASSCAP)/102400);
   
  mass = (cost * mass) / NOMMASS;

  if ( mass < 100 )  printf ("too cheap! cost=%ld, weight=%ld, genes=%ld, tc=%ld\n", cost,bug->pos[0].weight / 1024, bug->brain.ngenes , mass);

  bug->pos[0].weight -= mass;
  if ( bug->pos[0].weight <= 0 )
    bug->pos[0].weight = 1;

}


//
// Free the genes in this list
//
void free_genes( struct _gene *g )
{ struct _gene *n;

  while ( g != NULL )
    { n = g->next;
      free( g );
      g = n;
    }
}

//
// Free the genes from this brain
//
void free_brain( struct _bugbrain *bp )
{ long i;
  
  for ( i = 0 ; i < NDECISIONS ; i++ )
    { free_genes( bp->act[i].a );
      free_genes( bp->act[i].b );
      bp->act[i].a = NULL;
      bp->act[i].b = NULL;
    }
  bp->ngenes = 0;
}

//
// Determine ethnicity based on parents and position
//
void det_ethnicity( struct _ethnicity *offs, struct _ethnicity *mom, struct _ethnicity *dad, struct _pos *p )
{
  offs->r = (mom->r + dad->r) / 2;
  offs->g = (mom->g + dad->g) / 2;
  offs->b = (mom->b + dad->b) / 2;

  // Slowly assimilate
  switch ( (p->y * 3) / WORLD_Y )
    { case 0: // Blue sky land at the top
        if ( offs->r > 0 ) { offs->r--; offs->b++; }
        if ( offs->g > 0 ) { offs->g--; offs->b++; }
        while ( (offs->r + offs->g + offs->b) < ETHNIC_DUR ) offs->b++;
        break;
      case 1: // Redlands in the middle
        if ( offs->g > 0 ) { offs->g--; offs->r++; }
        if ( offs->b > 0 ) { offs->b--; offs->r++; }
        while ( (offs->r + offs->g + offs->b) < ETHNIC_DUR ) offs->r++;
        break;
      default:
      case 2: // Greenland at the bottom
        if ( offs->r > 0 ) { offs->r--; offs->g++; }
        if ( offs->b > 0 ) { offs->b--; offs->g++; }
        while ( (offs->r + offs->g + offs->b) < ETHNIC_DUR ) offs->g++;
        break;
    }
 
}


long countgenes( struct _gene *g )
{ long i;
  
  i = 0;
  while ( g != NULL )
    { g = g->next;
      i++;
    }
  return i;
}


struct _gene *cclp = NULL; // used in the recursive function copy_chromosome, should be initialized to NULL by calling function before call
//
// Copy a gene list, creating new genes and linking as we go
// 
struct _gene *copy_chromosome( struct _gene *g )
{ struct _gene *gp;

  if ( g != NULL )
    {  gp       = (Gene *)malloc( sizeof( struct _gene ) );       // allocate new structure
      *gp       = *g;                                            // copy the constants
       gp->next = NULL;                                         // maintain the list
       gp->prev = cclp;                                        // use the global to link to last allocated gene
       if ( cclp != NULL )                                    // fix next link in previous gene
         gp->prev->next = gp;
       cclp     = gp;                                       // ready for next gene
       gp->prod = copy_chromosome( g->prod );              // copy the branches
       gp->sum  = copy_chromosome( g->sum  );
       return gp;                                        // pointer to base of this sub-tree
    }
   else
    return NULL;
}

//
// Copy brain contents, from a to b
//
void copy_brain( struct _bugbrain *a, struct _bugbrain *b )
{ long  i;

  free_brain( b );
  *b = *a;  // copy the constants, including ngenes
  // now, do the linked lists
  for ( i = 0 ; i < NDECISIONS ; i++ )
    { cclp = NULL; b->act[i].a = copy_chromosome( a->act[i].a );
      cclp = NULL; b->act[i].b = copy_chromosome( a->act[i].b );
    }

}

long genesdropped;  // global counter for the recursive function disposebranch

//
// Prune the chromosome starting at g
// dispose of all prod and sum subterms
// and keep the next prev list maintained
// 
// note that g MUST NOT be the first gene in the tree (prev MUST NOT == NULL)
//
void disposebranch( struct _gene *g )
{ if ( g->prev == NULL )
    return; 


  if ( g->prod != NULL )
    disposebranch( g->prod );
  if ( g->sum != NULL )
    disposebranch( g->sum );
  
  if ( g->next != NULL )     // fix the list
    g->next->prev = g->prev;
  g->prev->next = g->next;
  free( g );
  genesdropped++;            // keep track
}



//
// Make a gene value tweak - or two or more
//
void tweakgene( struct _gene *g )
{ long d,r;

  r = 1 + limitedrandom( 255 );  // 50% chance of 1 tweak, 25% chance of 2 tweaks, 12% chance of 3 tweaks, 6% chance of 4....

  while ( r < 256 )
    {

          switch ( limitedrandom( 4 ) )
            { case 0: // Change gene type to one of the other gene types - note that the other values will either become active or dormant based on start and end types
                g->tp += limitedrandom( 4 ) + 1;
                if ( g->tp > 5 ) g->tp -= 5;
                break;
               
              case 1: // Change the sense index
                d = limitedrandom( NSENSES + 6 ) - 3;  // give slight preference to near 3 choices
                if ( d == 0 ) d = 6;                  // no change not allowed
                g->si += d;
                if ( g->si < 0         ) g->si += NSENSES;
                if ( g->si > NSENSES-1 ) g->si = g->si % NSENSES;
                break;
              
              case 2: 
                d = 1024 + limitedrandom( 256 ) - 128;
                g->c1 = (g->c1 * d) / 1024 + limitedrandom( 128 ) - 64;
                break;

              case 3: 
                d = 1024 + limitedrandom( 256 ) - 128;
                g->c2 = (g->c2 * d) / 1024 + limitedrandom( 128 ) - 64;
                break;
            }                
      r *= 2;
    }

}


//
// Make a number of mutations
//
void mutatebrain( struct _bugbrain *brain )
{ long n,c,s,d,r;
  struct _gene *g; 
  struct _gene *g2; 
  struct _gene *gn; 
  struct _ethnicity *ep;

  r = 1 + limitedrandom( 16383 );  // Up to 14 mutations, very low chance of higher numbers though

  while ( r < 16384 )  // 50% chance 1, 25% chance 2, etc...
    {

  n = limitedrandom( NDECISIONS + 1 );

  if ( n == NDECISIONS )
    { // mess with divide
      brain->divide += limitedrandom( 3 ) - 1; // Keep the change small 
      if ( brain->divide > 7 ) brain->divide = 6;  // bounce off the edges
      if ( brain->divide < 2 ) brain->divide = 3;
    }
   else
    { // n determines the chromosome number
      if ( limitedrandom( 2 ) ) // determine chain a or b
        { g  =   brain->act[n].a;
          ep = &(brain->act[n].ea);
        }
       else
        { g  =   brain->act[n].b;
          ep = &(brain->act[n].eb);
        }
      *ep = brain->eth;        // Record the bug that had this mutation for coloring purposes in the report
      g2 = g;                  // head of chain, for later work
      c = countgenes( g );
      c = limitedrandom( c );  // Choose a gene to act on
      while ( c > 0 )
        { g = g->next;
          c--;
        }
      if ( limitedrandom( 2 ) )
        { tweakgene( g );
        } 
       else
        { if ( limitedrandom( 4 ) != 0 )
            { // add a gene - a copy of the randomly selected g
              d = 0;
              while ( !d )  // randomly traverse the prod sum tree to an empty end node
                { s = limitedrandom( 2 );
                  if ( s )
                    { if ( g2->prod == NULL )
                        d = 1;
                       else
                        g2 = g2->prod;
                    }
                   else
                    { if ( g2->sum == NULL )
                        d = 1;
                       else
                        g2 = g2->sum;
                    }
                }
              gn = (Gene *)malloc( sizeof( struct _gene ) );
              brain->ngenes++;
              if ( s )
                g2->prod = gn;
               else
                g2->sum  = gn;
              *gn = *g;          // get the coefficients
              gn->sum  = NULL;  // fix up the pointers
              gn->prod = NULL;
              gn->next = NULL;
              while ( g->next != NULL )  // traverse to end of the list starting at g
                g = g->next;
               g->next = gn;
              gn->prev = g;
              if ( limitedrandom( 2 ) )   // 50/50 chance the new gene will be tweaked
                tweakgene( gn );
            }
           else
            { // prune the chromosome at g, if it has any prod or sum children
              // note that this will never clip off the first gene in the chain
              if ( g->prod != NULL )
                { if ( g->sum != NULL )
                    s = limitedrandom( 2 ); // both present, choose randomly
                   else
                    s = 1;                  // only prod
                }
               else
                { if ( g->sum != NULL )
                    s = 0;                  // only sum
                   else
                    s = 2;
                }
              genesdropped = 0;
              if ( s == 0 ) { disposebranch( g->sum  ); g->sum  = NULL; }
              if ( s == 1 ) { disposebranch( g->prod ); g->prod = NULL; }
              brain->ngenes -= genesdropped;
    }   }   }

    r *= 2;

    }
   
}
              
                       
//
// This bug has died - turn its mass into food and free its allocated structures
// 
void kill_bug( struct _bugdata *bug )
{ long x,y;

  x = bug->pos[0].p.x;
  y = bug->pos[0].p.y;

  if ( bug == nextglobalbug )
    nextglobalbug = bug->next;

  world[x][y].food += bug->pos[0].weight;
  world[x][y].bug   = NULL;

  free_brain( &(bug->brain)     );
  free_brain( &(bug->matebrain) );

  // Remove the bug from the list
  buglist.n_bugs--;
  if ( buglist.n_bugs > 0 )
    { if ( bug->prev == NULL )       // Was first in list
        {
          buglist.first   = bug->next;
          if ( buglist.first == NULL )   // No more bugs!
            buglist.endlist = NULL;
           else
            bug->next->prev = NULL;
        }
       else
        { if ( bug->next == NULL )  // Was last in list
            { bug->prev->next = NULL;
              buglist.endlist = bug->prev;
            }
           else
            { bug->prev->next = bug->next; // Was in middle of list
              bug->next->prev = bug->prev;
    }   }   }

  free( bug );
}

//
// bug move - act, do all energy accounting, mate and fight resolution, etc.
//
void bug_move( struct _bugdata *bug )
{             long  i,j,mass,face,ngenes;
       struct _pos  p;
   struct _bugdata *defender;
   struct _bugdata *offspring;


  gather_senses( bug );

  // shift history back 1 space
  for ( i = POSHISTORY-1 ; i > 0 ; i-- )
    bug->pos[i] = bug->pos[i-1]; 

  i = bugdecide( &(bug->brain) ); 
  bug->pos[0].act = i;
  switch( i )
    { case ACTSLEEP:
        costcalc( COSTSLEEP, bug );
        break;

      case ACTEAT:
        mass = ( bug->pos[0].weight * EATLIMIT ) / 1024;           // limit food intake to EATLIMIT% of body weight
        if ( mass > world[bug->pos[0].p.x][bug->pos[0].p.y].food ) // and amount of food available in this cell
          { bug->pos[0].weight -= (mass - world[bug->pos[0].p.x][bug->pos[0].p.y].food); // Penalty for overeating
            mass = world[bug->pos[0].p.x][bug->pos[0].p.y].food;
          }
        bug->pos[0].weight += mass;
        world[bug->pos[0].p.x][bug->pos[0].p.y].food -= mass;      // move the mass from land to bug
        costcalc( COSTEAT, bug );                                  // and pay for lunch
        break;

      case ACTTURNCW:
        if ( bug->pos[0].face < 3 )
          bug->pos[0].face += 1;
         else
          bug->pos[0].face = -2;
        costcalc( COSTTURN, bug );
        break;

      case ACTTURNCCW:
        if ( bug->pos[0].face > -2 )
          bug->pos[0].face -= 1;
         else
          bug->pos[0].face = 3;
        costcalc( COSTTURN, bug );
        break;

      case ACTMOVE:
        bug->moves++;
        hist[today % LHIST].movement++;
        p = bug->pos[0].p;
        hexmove( &p , bug->pos[0].face );     // the destination
        defender = world[p.x][p.y].bug;       // check for squatters
        costcalc( COSTMOVE, bug );            // pay for the move
        if ( bug->pos[0].weight < 0 ) bug->pos[0].weight = 0;  
        if ( defender != NULL )
          { if ( safety ) break; // No kills while safety is on

            hist[today % LHIST].collisions++;
            mass = defender->pos[0].weight;
	    i = defender->pos[0].face - bug->pos[0].face;       // deermine relative facing
            while ( i < -2 ) i += 6; while ( i >  3 ) i -= 6;  // correct for overflow
            switch ( i )
              { case 0:
                  mass *= (defender->defends / 2) + 1;  // Experience points
                  mass /= 128;                         // Head on, advantage to the defender
                  break;
                case  1:
                case -1:                                // Oblique from the front, no adjustment
                  mass *= (defender->defends / 4) + 1; // Experience points
                  mass /= 1024;
                  break;
                case  2:
                case -2:
                  mass *= (defender->defends / 8) + 1; // Experience points
                  mass /= 8192;                       // oblique from the rear, advantage to the attacker
                  mass -= bug->kills;                // Experience advantage to the attacker
                  break;
                case 3:
                  mass /= 65536;                    // from the rear, BIG advantage to the attacker - experience doesn't help
                  mass -= bug->kills * bug->kills; // BIG Experience advantage to the attacker
                  break;
              }
            if ( mass < 0 ) mass = 0;
            if ( limitedrandom( mass + (bug->pos[0].weight / 1024) ) > mass )
              { // victory
                bug->kills++;
                kill_bug( defender );
                world[p.x][p.y].bug = bug;                                 // move in
                world[bug->pos[0].p.x][bug->pos[0].p.y].bug = NULL;        // erase link from where we were
                bug->pos[0].p = p;                                         // locate ourselves
                costcalc( COSTFIGHT, bug );         // pay for the fight
              }
             else
              { // defeat
                defender->defends++;
                world[p.x][p.y].food += bug->pos[0].weight;
                bug->pos[0].weight = 0;                                         // note bring food to the defender
                kill_bug( bug ); 
                bug = NULL;
                world[p.x][p.y].bug = defender;     // fix the link to the victor
                // shift history of defender back 1 space
                for ( i = POSHISTORY-1 ; i > 0 ; i-- )
                  defender->pos[i] = defender->pos[i-1]; 
                defender->pos[0].act = ACTDEFEND;
              }
          }
         else
          { // Do the move! - no defender
            world[p.x][p.y].bug = bug;                                 // move in
            world[bug->pos[0].p.x][bug->pos[0].p.y].bug = NULL;        // erase link from where we were
            bug->pos[0].p = p;                                         // locate ourselves
          }      
        break;

      case ACTMATE:
        p = bug->pos[0].p;
        hexmove( &p , bug->pos[0].face );      // location of the potential mate
        if ( world[p.x][p.y].bug != NULL )
          { // gather_senses( world[p.x][p.y].bug ); decided to view the world through the suitor's eyes, but make own decision // senses to make the decision - might add a few for mate identification, kinda confusing as it is
            if ( evaluate_gene( world[p.x][p.y].bug->brain.act[RESPONSEMATE].a ) +     
                 evaluate_gene( world[p.x][p.y].bug->brain.act[RESPONSEMATE].b ) > 0 )   // NOTE!!! RESPONSEMATE IS A WEIRD DOUBLE ACTING CHROMOSOME - different from the others
              { // success, swap brains for later use during division
                if ( bug->matebrain.eth.uid != world[p.x][p.y].bug->brain.eth.uid )
                  bug->mate_success++;
                 else
                  bug->mate_repeat++;
                if ( bug->brain.eth.uid != world[p.x][p.y].bug->matebrain.eth.uid )
                  world[p.x][p.y].bug->mate_success++;
                 else
                  world[p.x][p.y].bug->mate_repeat++;
                copy_brain( &(world[p.x][p.y].bug->brain), &(bug->matebrain) );
                copy_brain( &(bug->brain), &(world[p.x][p.y].bug->matebrain) );
                // shift history of mate back 1 space
                for ( j = POSHISTORY-1 ; j > 0 ; j-- )
                  world[p.x][p.y].bug->pos[j] = world[p.x][p.y].bug->pos[j-1]; 
                world[p.x][p.y].bug->pos[0].act = ACTMATED;
                                bug->pos[0].act = ACTMATED;
              }
             else
              bug->mate_fails++;
          }
         else
          bug->mate_fails++;
        costcalc( costmate, bug );
        break;
             
      case ACTDIVIDE:
        if ( forcemate & 0x10 )  // enforce agediv
          if ( bug->birthday + agediv > today )
            { if ( forcemate & 0x40 )
                bug->pos[0].weight /= bug->brain.divide;
              if ( forcemate & 0x20 )
                bug->pos[0].weight -= COSTDIVIDE;  // Higher cost for higher forcemate levels
              if ( bug->pos[0].weight < DIETHIN )
                bug->pos[0].weight = DIETHIN;
              costcalc( COSTSLEEP, bug );
              break;
            }
        if ( forcemate & 0x01 )  // enforce "forcemate before div"
          if ( bug->brain.eth.uid == bug->matebrain.eth.uid )
            { if ( forcemate & 0x08 )
                bug->pos[0].weight /= bug->brain.divide;
              if ( forcemate & 0x04 )
                bug->pos[0].weight -= COSTDIVIDE;  // Higher cost for higher forcemate levels
              if ( bug->pos[0].weight < DIETHIN )
                bug->pos[0].weight = DIETHIN;
              costcalc( COSTSLEEP, bug );
              break;
            }
        mass = (bug->pos[0].weight / bug->brain.divide) - COSTDIVIDE; 
        bug->pos[0].weight = mass;
        if ( mass < DIETHIN )
          break;
 
        for ( i = 1 ; i < bug->brain.divide ; i++ )  // # of children
          { p    = bug->pos[0].p;
            face = bug->pos[0].face;
            switch( i ) // rearranged.
              { case 1: face = face + 3; break;
                case 2: face = face - 2; break;
                case 3: face = face + 2; break;
                case 4: face = face - 1; break;
                case 5: face = face + 1; break;
                case 6: break;
              }
            hexmove( &p , face );

            if ( world[p.x][p.y].bug == NULL )  // If space not empty, offspring is never born
              {
            bug->offspring++;
            hist[today % LHIST].births++;

            // Create new life - a new bug, with traits of brain and matebrain - and possibly a mutation 
            offspring  = (Bugdata *)malloc( sizeof( struct _bugdata ) );

            world[p.x][p.y].bug   = offspring;

            buglist.endlist->next = offspring;           // It is logical to assume that the list is not empty
                  offspring->next = NULL;
                  offspring->prev = buglist.endlist;
                  buglist.endlist = offspring;
            buglist.n_bugs++;
            offspring->brain.eth.uid = idcounter++;        // The bug social security number
            if ( bug->brain.generation > bug->matebrain.generation )
              offspring->brain.generation = bug->brain.generation + 1;
             else
              offspring->brain.generation = bug->matebrain.generation + 1;
            offspring->birthday     = today;
            offspring->kills        = 
            offspring->defends      =
            offspring->mate_success = 
            offspring->mate_repeat  =
            offspring->mate_fails   =
            offspring->moves        =
            offspring->offspring    = 0;
            offspring->brain.family[0] = bug->brain.eth;
            offspring->brain.family[1] = bug->matebrain.eth;
            det_ethnicity( &(offspring->brain.eth), &(bug->brain.eth), &(bug->matebrain.eth), &p );
            j = 2;
            while ( j + 1 < FAMHIST )
              { offspring->brain.family[ j ] = bug->brain.family[(j/2)-1];
                offspring->brain.family[j+1] = bug->matebrain.family[(j/2)-1];
                j += 2;
              }

            for ( j = POSHISTORY-1 ; j >= 0 ; j-- )
              { offspring->pos[j].p      = p;
                offspring->pos[j].face   = face;
                offspring->pos[j].act    = ACTSLEEP;
                offspring->pos[j].weight = mass;
              }
            ngenes = 0;
            for ( j = 0 ; j < NDECISIONS ; j++ )
              { if ( limitedrandom( 2 ) )  // a haploid chromosome comes from parent
                  { cclp = NULL; offspring->brain.act[j].a = copy_chromosome( bug->brain.act[j].a ); offspring->brain.act[j].ea = bug->brain.act[j].ea; }
                 else
                  { cclp = NULL; offspring->brain.act[j].a = copy_chromosome( bug->brain.act[j].b ); offspring->brain.act[j].ea = bug->brain.act[j].eb; }

                if ( limitedrandom( 2 ) ) // b haploid chromosome comes from mate
                  { cclp = NULL; offspring->brain.act[j].b = copy_chromosome( bug->matebrain.act[j].a ); offspring->brain.act[j].eb = bug->matebrain.act[j].ea; }
                 else
                  { cclp = NULL; offspring->brain.act[j].b = copy_chromosome( bug->matebrain.act[j].b ); offspring->brain.act[j].eb = bug->matebrain.act[j].eb; }

                ngenes += countgenes( offspring->brain.act[j].a );
                ngenes += countgenes( offspring->brain.act[j].b );
                offspring->matebrain.act[j].a = NULL;
                offspring->matebrain.act[j].b = NULL;
              }
            offspring->brain.ngenes = ngenes;
            if ( limitedrandom( 2 ) )
              offspring->brain.divide = bug->brain.divide;
             else
              offspring->brain.divide = bug->matebrain.divide;

            offspring->brain.expression = limitedrandom( 256 );  // 256 = 2^NDECISIONS, tempting to make this environmentally dependent...

            copy_brain( &(offspring->brain), &(offspring->matebrain) );  // also copies the ngenes and ndivide values

            if ( limitedrandom( 4 ) == 0 )
              mutatebrain( &(offspring->matebrain) ); // asexual reproduction has a good chance of catching a mutation
            if ( limitedrandom( 8 ) == 0 )
              mutatebrain( &(offspring->brain) );    // even 100% sexual reproducers have a chance of mutation
              }


          }
        if ( forcemate & 0x02 )
          bug->matebrain.eth.uid = bug->brain.eth.uid;
        break;
    }

  if ( bug != NULL )   // could have been killed in a fight
    if ( bug->pos[0].weight < DIETHIN )
      { kill_bug( bug ); // too thin to live, feed the grass
        hist[today % LHIST].starvations++;
      }
        
}

//
// Move all bugs
//
void move_bugs( void )
{
  struct _bugdata *bug;

  bug = buglist.first;
  while ( bug != NULL )
    { nextglobalbug = bug->next;  // Incase this one is killed during the move
      bug_move( bug );
      bug = nextglobalbug;
    }

}

//
// Simple add gene function for bug_one - cannot build trees, only a chain
// gene created gets put at the head of the chain and pointer returned
//
struct _gene *add_gene( long tp, long si, long c1, long c2, struct _gene *og, long p )
{ struct _gene *ng;
  
  ng = (Gene *)malloc( sizeof( struct _gene ) );
  ng->tp = tp;
  ng->si = si;
  ng->c1 = c1;
  ng->c2 = c2;
  ng->next = og;
  ng->prev = NULL;
  ng->prod = NULL;
  ng->sum  = NULL;

  if ( og != NULL )
    { og->prev = ng;
      if ( p == 0 )
        ng->sum  = og;
       else
        ng->prod = og;
    }

  return ng;
}
  
//
// Model bugs
//
// 0a>[ 0,1,055,   226,  1343,p- ,s- ]
// 1a>[ 0,3,057,  1216,  1084,p- ,s 1][ 1,5,057,  1216,  1084,p- ,s- ]
// 2a>[ 0,4,020,   629,   830,p 1,s- ][ 1,5,042,   555,   668,p 2,s- ][ 2,4,025,   636,   829,p- ,s- ]
// 3b>[ 0,1,011,   319,  1426,p- ,s- ]
// 4b>[ 0,3,000,   173,   -53,p- ,s 1][ 1,3,058,  4273,  2187,p- ,s- ]
// 5b>[ 0,2,055,  1421,   456,p 1,s- ][ 1,2,013,   734,   101,p- ,s- ]
// 6t>[ 0,5,024,  1754,  2063,p- ,s 1][ 1,4,015,  1231,   936,p 2,s- ][ 2,3,046,  1071,  2179,p- ,s- ]
// 7b>[ 0,3,011,   -50,   591,p- ,s- ]

// 0a>[ 0,3,055,   363,  1530,p- ,s- ]
// 1a>[ 0,3,057,  1203,  1056,p- ,s- ]
// 2a>[ 0,4,020,   615,   830,p 1,s- ][ 1,5,042,   525,   697,p 2,s- ][ 2,5,025,   658,   764,p- ,s- ]
// 3b>[ 0,1,016,   341,  1570,p- ,s- ]
// 4b>[ 0,3,000,   226,   -76,p- ,s 1][ 1,3,058,  3944,  2187,p- ,s- ]
// 5a>[ 0,2,055,  1339,   567,p 1,s- ][ 1,2,013,   785,   101,p- ,s- ]
// 6t>[ 0,5,024,  1901,  2063,p- ,s 1][ 1,3,045,  1170,   845,p 2,s- ][ 2,3,055,  1071,  1916,p- ,s- ]
// 7a>[ 0,3,051,   -79,   546,p- ,s- ]




//
// Create the original bug
//
void bug_one( void )
{ struct _bugdata *bug;
  struct     _pos  p;
             long  i;

  bug = (Bugdata *)malloc( sizeof( struct _bugdata ) );
  p.x = WORLD_X / 2;
  p.y = WORLD_Y / 2;
  world[p.x][p.y].bug = bug;
  
  for ( i = 0 ; i < FAMHIST ; i++ )
    { bug->brain.family[i].uid = -1;             // Unknown history
      bug->brain.family[i].r   = ETHNIC_DUR / 8;
      bug->brain.family[i].g   = ETHNIC_DUR / 8;
      bug->brain.family[i].b   = ETHNIC_DUR / 8;
    }
  bug->brain.eth.uid = idcounter++;        // The bug social security number
  bug->birthday       = today;
  bug->kills          = 0;
  bug->defends        = 0;
  bug->mate_success   = 0;
  bug->mate_fails     = 0;
  bug->moves          = 0;
  for ( i = POSHISTORY-1 ; i >= 0 ; i-- )
    { bug->pos[i].p      = p;
      bug->pos[i].face   = DIR_E;
      bug->pos[i].act    = ACTSLEEP;
      bug->pos[i].weight = DIETHIN * 256;  // Fat, happy, ready to make children
    }
  bug->prev       = NULL;
  bug->next       = NULL;
  buglist.first   = bug;
  buglist.endlist = bug;
  buglist.n_bugs  = 1;

  bug->brain.generation = 0;
  bug->brain.divide     = 3;  // 2 offspring per division - parent just loses weight

  bug->brain.eth.r = ETHNIC_DUR;
  bug->brain.eth.g = 0;
  bug->brain.eth.b = 0;

  for ( i = 0 ; i < NDECISIONS ; i++ )
    { switch ( i )
        { case 0:
          default:                 // shouldn't be used... 
            bug->brain.act[i].a = add_gene( 1, 55,  26,  363, NULL, 0 );
            bug->brain.act[i].b = add_gene( 1, 55,  63, 1530, NULL, 0 );
            break;

          case 1:
            bug->brain.act[i].a = add_gene( 5, 57, 1216, 1084, NULL               , 0 );
            bug->brain.act[i].a = add_gene( 3, 57, 1216, 1084, bug->brain.act[i].a, 0 );
            bug->brain.act[i].a = add_gene( GENECONST, NSENSECELLS + 1, 1500, 1048, bug->brain.act[i].a, 1 );
            bug->brain.act[i].b = add_gene( 3, 57, 1203, 1056, NULL               , 0 );
            bug->brain.act[i].b = add_gene( GENECONST, NSENSECELLS + 1, 2000, 1048, bug->brain.act[i].b, 1 );
            break;

          case 3:
            bug->brain.act[i].a = add_gene( GENELIMIT, SENSESELF + i, 100, 1000, NULL, 0 );
            bug->brain.act[i].b = add_gene( GENELIMIT, SENSESELF + i, 510,  514, NULL, 0 );
            break;

          case 2:
            bug->brain.act[i].a = add_gene( GENELIMIT, SENSESELF + i,  50, 1200, NULL, 0 );
            bug->brain.act[i].b = add_gene( GENELIMIT, SENSESELF + i, 760,  776, NULL, 0 );
            break;

          case 4:
            bug->brain.act[i].a = add_gene( 3, 58, 4274, 2187, NULL               , 0 );
            bug->brain.act[i].a = add_gene( 3,  0,  173,  -53, bug->brain.act[i].a, 0 );
            bug->brain.act[i].a = add_gene( GENECONST, NSENSECELLS + 1, 1500, 1048, bug->brain.act[i].a, 1 );
            bug->brain.act[i].b = add_gene( 3, 58, 3944, 2187, NULL               , 0 );
            bug->brain.act[i].b = add_gene( 3,  0,  226,  -76, bug->brain.act[i].b, 0 );
            bug->brain.act[i].b = add_gene( GENECONST, NSENSECELLS + 1, 2000, 1048, bug->brain.act[i].b, 1 );
            break;

          case 5:
            bug->brain.act[i].a = add_gene( 2, 13,  734,  101, NULL               , 0 );
            bug->brain.act[i].a = add_gene( 2, 55, 1421,  456, bug->brain.act[i].a, 1 );
            bug->brain.act[i].b = add_gene( 2, 13,  785,  101, NULL               , 0 );
            bug->brain.act[i].b = add_gene( 2, 55, 1339,  567, bug->brain.act[i].b, 1 );
            break;

          case 6:
            bug->brain.act[i].a = add_gene( GENELIMIT, SPAWNWEIGHTNORM, 1200, 3000, NULL               , 1 );
            bug->brain.act[i].a = add_gene( GENECONST, NSENSECELLS + 1, 3500, 1048, bug->brain.act[i].a, 1 );
            bug->brain.act[i].b = add_gene( GENELIMIT, SPAWNWEIGHTNORM, 1800, 1850, NULL               , 1 );
            bug->brain.act[i].b = add_gene( GENECONST, NSENSECELLS + 1, 4000, 1048, bug->brain.act[i].b, 1 );
            break;

          case 7:
            bug->brain.act[i].a = add_gene( 3, 11,  -50,  591, NULL               , 0 );
            bug->brain.act[i].b = add_gene( 3, 51,  -79,  546, NULL               , 0 );
            break;
        }      
      bug->brain.ngenes += countgenes( bug->brain.act[i].a );
      bug->brain.ngenes += countgenes( bug->brain.act[i].b );
      bug->brain.act[i].ea = bug->brain.eth;
      bug->brain.act[i].eb = bug->brain.eth;
      bug->matebrain.act[i].a = NULL;
      bug->matebrain.act[i].b = NULL;
    }

  copy_brain( &(bug->brain), &(bug->matebrain) );  // also copies the ngenes and ndivide values
  mutatebrain( &(bug->matebrain) );                // asexual reproduction always has a 50% chance of catching a mutation

}

char *font_color( struct _ethnicity *e )
{ static char str[40];

  sprintf( str, "<font color=\"#%02x%02x%02x\">",(255 * ((short)e->r)) / ETHNIC_DUR
                                                ,(255 * ((short)e->g)) / ETHNIC_DUR 
                                                ,(255 * ((short)e->b)) / ETHNIC_DUR );
  return str;
}

void chromosome_dump( FILE *fp, struct _gene *g )
{ struct _gene *p;
          long  i;

  p = g;
  i = 0;
  while ( p != NULL )
    { p->tp += i;      // markup for indexing
      p = p->next;
      i += 1024;
    }

  p = g;
  while ( p != NULL )
    { if ( ( (p->tp / 1024) % 4 == 0 ) && ( p->tp > 1536 ) ) fprintf( fp,"\r\n            " );
      fprintf( fp, "[%2ld,%ld,%03ld,%6ld,%6ld,", p->tp / 1024,p->tp % 1024,p->si,p->c1,p->c2 );
      if ( p->prod == NULL )
        fprintf( fp, "p- ," );
       else
        fprintf( fp, "p%2ld,", p->prod->tp / 1024 );

      if ( p->sum == NULL )
        fprintf( fp, "s- ]" );
       else
        fprintf( fp, "s%2ld]", p->sum->tp / 1024 );
      p = p->next;
    }
 
  p = g;
  while ( p != NULL )
    { p->tp = p->tp % 1024;      // markup for indexing
      p = p->next;
    }
}

//
// Finding the bug that does the most with the least number of genes
//
long lean_genes( struct _bugdata *bp )
{ long p;

  // Age is an implied factor here
  p = (1024 * bp->moves) / bp->brain.ngenes;
  p = (p * (bp->mate_success + 1) ) / bp->brain.ngenes;
  p = (p * bp->offspring * bp->offspring) / bp->brain.ngenes;

  return p;
}


//
// Complex formula for the "killer bug"
//
long slasher( struct _bugdata *bp )
{ long age,offspring,moves,kills,defends,mates,p;

  age       = today - bp->birthday;
  offspring = bp->offspring;
  moves     = bp->moves;
  kills     = bp->kills;
  defends   = bp->defends;
  mates     = bp->mate_success;

  p = kills * kills * kills * (offspring * 4 + mates + 1) * 1024;
  p = p / (age * moves + 1024);

  return p;
}


void bug_dump( FILE *fp, struct _bugdata *bug )
{ long i,x;

  if ( bug == NULL )
    return;
  fprintf( fp, "%sBug #%ld, generation %ld, %ld turns old, %ld genes, %ld mass, [%ld,%ld] current pos<br>\r\n", font_color( &(bug->brain.eth) ), bug->brain.eth.uid, bug->brain.generation, today - bug->birthday, bug->brain.ngenes, bug->pos[0].weight / 1024, bug->pos[0].p.x, bug->pos[0].p.y );
  fprintf( fp, "%ld moves, %ld kills, %ld defs, %ld M+, %ld Mr, %ld M-, %ld/(%ld) offs, %ld lean, %ld slasher<br>\r\n", bug->moves, bug->kills, bug->defends, bug->mate_success, bug->mate_repeat, bug->mate_fails, bug->offspring, bug->brain.divide, lean_genes( bug ), slasher( bug ) );

  fprintf( fp, "<font size=-2><PRE>\r\nFamily History: \r\n" );
  for ( i = 0 ; i < FAMHIST ; i++ )
    { fprintf( fp, "%s%7ld</font> ", font_color( &(bug->brain.family[i]) ), bug->brain.family[i].uid );
      if (( i ==  1 ) ||
          ( i ==  5 ) ||
          ( ((i - 13) % 16) == 0 ))
        fprintf( fp,"\r\n" );
    }
  fprintf( fp, "\r\n" );

  x = 1;

  for ( i = 0 ; i < NDECISIONS ; i++ )
    { fprintf (fp,"%s",font_color( &(bug->brain.act[i].ea) ) );
      if ( (bug->brain.expression & x) != 0 )
        fprintf( fp, "%2lda%8ld>", i, bug->brain.act[i].ea.uid );
       else
        fprintf( fp, "%2lda%8ld-", i, bug->brain.act[i].ea.uid );
      chromosome_dump( fp, bug->brain.act[i].a );
      fprintf (fp,"</font>%s",font_color( &(bug->brain.act[i].eb) ) );
      if ( (bug->brain.expression & x) == 0 )
        fprintf( fp, "\r\n%2ldb%8ld>", i, bug->brain.act[i].eb.uid );
       else
        fprintf( fp, "\r\n%2ldb%8ld-", i, bug->brain.act[i].eb.uid );
      chromosome_dump( fp, bug->brain.act[i].b );
      fprintf( fp, "</font>\r\n\r\n" );

      x *= 2;
    }
  fprintf( fp, "</PRE></font></font>\r\n" );
}


void bug_report( char *fname, char *iname )
{ struct _bugdata *bp;
  struct _bugdata *bm;
             FILE *fp;
             long  genesum[NDECISIONS];
             long  i,x,y;

  fp = fopen( fname, "wb");
  
  fprintf( fp,"<HTML><HEAD><TITLE>Bug Report Year %6.2f</TITLE></HEAD><BODY TEXT=\"#C0C0C0\" BGCOLOR=\"#000000\">\r\n", 
         ((float)today)/((float)SEASONLENGTH) );

  fprintf( fp,"<CENTER><H1>Bug Report</H1><H2>Year %6.2f</H2><img src=\"%s\"><br></CENTER>\r\n",
         ((float)today)/((float)SEASONLENGTH), iname );

  fprintf( fp, "%ld Days elapsed<br>%ld Bugs living, %4.1f%% space consumed<br>%ld bugs born throughout history<br>%5.0f Food per cell, on average<br>%5.0f mass of average bug<br>%ld target population<br>%6.2f genes in average bug:<br>\r\n", 
                today, buglist.n_bugs, ((float)(buglist.n_bugs * 100))/((float)(WORLD_X * WORLD_Y)), idcounter, (float)totalfood/((float)WORLD_X * WORLD_Y), (float)totalbug/((float)buglist.n_bugs*1024),target_pop,((float)genecount)/((float)buglist.n_bugs) ); 

  for ( i = 0 ; i < NDECISIONS ; i++ )
    genesum[i] = 0;
  for ( x = 0 ; x < WORLD_X ; x++ )
    for ( y = 0 ; y < WORLD_Y ; y++ )
      { if ( world[x][y].bug != NULL )
          for ( i = 0 ; i < NDECISIONS ; i++ )
            genesum[i] += countgenes( world[x][y].bug->brain.act[i].a ) + 
                          countgenes( world[x][y].bug->brain.act[i].b );
      }
  y = 0;
  for ( i = 0 ; i < NDECISIONS ; i++ )
    y += genesum[i];
  for ( i = 0 ; i < NDECISIONS ; i++ )
    fprintf( fp, "%4.1f%% in chromosome %ld<br>",((float)(100 * genesum[i]))/((float)y), i );

  fprintf( fp, "%4.1f Gene Knee<br>", sqrt( ((float)geneknee2) ) );
  fprintf( fp, "%3ld Min Age of Division, materule: %02lx food factor %5.3f<br>", agediv, forcemate, foodhump );

  fprintf( fp, "<br>Exceptional bug reports:<br><br>" );


  fprintf( fp, "Oldest\r\n" );
  bug_dump( fp, buglist.first );

  fprintf( fp, "Newest\r\n" );
  bug_dump( fp, buglist.endlist );

  fprintf( fp, "Median\r\n" );
  bp = buglist.first;
  i = buglist.n_bugs / 2;
  while ( i > 0 )
    { bp = bp->next;
      i--;
    }
  bug_dump( fp, bp );
  

  fprintf( fp, "Most kills\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->kills > bm->kills )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Most moves\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->moves > bm->moves )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Most defends\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->defends > bm->defends )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Most offspring\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->offspring > bm->offspring )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Lowest generation\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->brain.generation < bm->brain.generation )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Highest generation\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->brain.generation > bm->brain.generation )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Least genes\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->brain.ngenes < bm->brain.ngenes )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Most genes\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->brain.ngenes > bm->brain.ngenes )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Heaviest\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( bp->pos[0].weight > bm->pos[0].weight )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Lean Genes\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( lean_genes( bp ) > lean_genes( bm ) )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp, "Slasher Prize\r\n" );
  bp = bm = buglist.first;
  while ( bp != NULL )
    { if ( slasher( bp ) > slasher( bm ) )
        bm = bp;
       bp = bp->next;
    }
  bug_dump( fp, bm ); 

  fprintf( fp,"</BODY></HTML>\r\n");

  fclose( fp );
}

void image_plot( gdImagePtr im, char *fn )
{           long  x,y,i,r,g,b,c,bugs,age,mass,kills,genes,maxbugs,maxage,maxkills,maxbd,maxmass,maxgenes,minmass,mingenes;
            long  lastage,lastbugs,lastmass,lastkills,lastgenes,poppct;
            long  actsum[NACT];
            FILE *jpegout; //output file
 struct _bugdata *bug;

          c = gdTrueColor( 0,0,0 );
          gdImageFilledRectangle(im, 0, 0, WORLD_X + SIDEBAR, WORLD_Y + BOTTOMBAR, c); // Erase image


          if ( (today % SEASONLENGTH) > 1024 )
            {
// Plot the map
          for ( i = POSHISTORY-1 ; i >= 0 ; i-- )
            { bug = buglist.first;
              while ( bug != NULL )
                { r = (255 * (((short)bug->brain.eth.r) * ( POSHISTORY - i )) / POSHISTORY) / ETHNIC_DUR;
                  g = (255 * (((short)bug->brain.eth.g) * ( POSHISTORY - i )) / POSHISTORY) / ETHNIC_DUR;
                  b = (255 * (((short)bug->brain.eth.b) * ( POSHISTORY - i )) / POSHISTORY) / ETHNIC_DUR;
                  c = gdTrueColor( r,g,b );
                  gdImageSetPixel( im,LEFTBAR + bug->pos[i].p.x,bug->pos[i].p.y,c );
                  bug = bug->next;
                }
            }
            }
           else
            {
// Old style
          for ( x = 0 ; x < WORLD_X ; x++ )
            for ( y = 0 ; y < WORLD_Y ; y++ )
               { 
                 if ( world[x][y].bug != NULL ) 
                   { r  = 255 + (world[x][y].bug->birthday - today) / 4; if ( r <   0 ) r =   0;
                     b  = (today - world[x][y].bug->birthday) / 16;      if ( b >  64 ) b =  64;
                     b += world[x][y].bug->pos[0].weight / 1536;         if ( b > 255 ) b = 255;
                   }
                  else
                   { r = b = 0; }
                 g = ( world[x][y].food * 192 ) / FOODCAP; if ( g > 255 ) g = 255;
                 c = gdTrueColor( r,g,b );
                 gdImageSetPixel( im,x + LEFTBAR,y,c );
               }
            }

          // Draw the bottom graph
          // do the autoranging
          maxbugs  = maxbd = maxmass = maxgenes = 1;
          minmass  = 0; // hist[today % LHIST].avgweight;
          mingenes = hist[today % LHIST].avggenes;
          y = WORLD_X + SIDEBAR;
          if ( y > today )
            y = today;
          for ( x = 0 ; x < y ; x++ )
            { if ( hist[(today - x) % LHIST].n_bugs > maxbugs )
                maxbugs = hist[(today - x) % LHIST].n_bugs;

              poppct = (1024 * hist[(today - x) % LHIST].n_bugs) / (WORLD_X  * WORLD_Y);

              if ( hist[(today - x) % LHIST].births > maxbd )  // births, starve+collide, and movement
                maxbd = hist[(today - x) % LHIST].births;      // all plot on the same scale
              if ( hist[(today - x) % LHIST].collisions + 
                   hist[(today - x) % LHIST].starvations > maxbd )
                maxbd = hist[(today - x) % LHIST].collisions
                      + hist[(today - x) % LHIST].starvations;
              if ( hist[(today - x) % LHIST].movement * poppct / 1024 > maxbd ) 
                maxbd = hist[(today - x) % LHIST].movement * poppct / 1024;

              if ( hist[(today - x) % LHIST].avgweight > maxmass )
                maxmass = hist[(today - x) % LHIST].avgweight;
              if ( hist[(today - x) % LHIST].avgfood > maxmass )
                maxmass = hist[(today - x) % LHIST].avgfood;

              if ( hist[(today - x) % LHIST].avgweight < minmass )
                minmass = hist[(today - x) % LHIST].avgweight;
              // if ( hist[(today - x) % LHIST].avgfood < minmass )
              //   minmass = hist[(today - x) % LHIST].avgfood;

              if ( hist[(today - x) % LHIST].avggenes > maxgenes )
                maxgenes = hist[(today - x) % LHIST].avggenes;
              if ( hist[(today - x) % LHIST].avggenes < mingenes )
                mingenes = hist[(today - x) % LHIST].avggenes;
            }
          if ( maxgenes == mingenes ) maxgenes++;
          if ( maxmass == minmass ) maxmass++;
          if ( maxbd == 0 ) maxbd++;
          // Plot the graph
          for ( x = 0 ; x < y ; x++ )
            { c = gdTrueColor( 255,255,255 );  // nbugs forms a white background
              gdImageLine( im, WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1, 
                               WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - (hist[(today - x) % LHIST].n_bugs * BOTTOMBAR) / maxbugs, c );
              if ( x > 0 )
                { c = gdTrueColor( 96,96,96 );  // genes are grey
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x + 1) % LHIST].avggenes - mingenes) * BOTTOMBAR) / (maxgenes - mingenes),
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x    ) % LHIST].avggenes - mingenes) * BOTTOMBAR) / (maxgenes - mingenes), c );

                  c = gdTrueColor( 0, 255, 0 );  // food is green
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x + 1) % LHIST].avgfood - minmass) * BOTTOMBAR) / (maxmass - minmass),
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x    ) % LHIST].avgfood - minmass) * BOTTOMBAR) / (maxmass - minmass), c );

                  c = gdTrueColor( 0, 0, 255 );  // bugs are blue
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x + 1) % LHIST].avgweight - minmass) * BOTTOMBAR) / (maxmass - minmass),
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x    ) % LHIST].avgweight - minmass) * BOTTOMBAR) / (maxmass - minmass), c );

                  poppct = (1024 * hist[(today - x) % LHIST].n_bugs) / (WORLD_X  * WORLD_Y);

                  c = gdTrueColor( 0,255,128 );  // movement is bright green
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x + 1) % LHIST].movement * poppct / 1024) * BOTTOMBAR) / maxbd,
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x    ) % LHIST].movement * poppct / 1024) * BOTTOMBAR) / maxbd, c );

                  c = gdTrueColor( 0,128,0 );  // starvations are dk green, plotted first so may be overwritten, and plotted on top of collisions as a total death measure
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x + 1) % LHIST].collisions + hist[(today - x + 1) % LHIST].starvations) * BOTTOMBAR) / maxbd,
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - ((hist[(today - x    ) % LHIST].collisions + hist[(today - x    ) % LHIST].starvations) * BOTTOMBAR) / maxbd, c );

                  c = gdTrueColor( 255,0,0 );  // collisions are red
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - (hist[(today - x + 1) % LHIST].collisions * BOTTOMBAR) / maxbd,
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - (hist[(today - x    ) % LHIST].collisions * BOTTOMBAR) / maxbd, c );

                  c = gdTrueColor( 255,0,255 );  // births are magenta
                  gdImageLine( im, WORLD_X + SIDEBAR - x    , WORLD_Y + BOTTOMBAR - 1 - (hist[(today - x + 1) % LHIST].births * BOTTOMBAR) / maxbd,
                                   WORLD_X + SIDEBAR - 1 - x, WORLD_Y + BOTTOMBAR - 1 - (hist[(today - x    ) % LHIST].births * BOTTOMBAR) / maxbd, c );

               }
            }

          // Now, need to compute and plot the activity ratios down the rightbar
            
          for ( y = 0 ; y < WORLD_Y ; y++ )
            { for ( r = 0 ; r < NACT ; r++ )
                actsum[r] = 0;
              c = 0;  // totalsum
              for ( x = 0 ; x < WORLD_X ; x++ )
                { if ( world[x][y].bug != NULL )
                    { g = world[x][y].bug->birthday;
                      b = 0;
                      while (( g < today ) && ( b < POSHISTORY ))
                        { actsum[ world[x][y].bug->pos[b].act ]++;
                          b++;
                          g++;
                          c++;
                        }
                    }
                }
              b = g = 0;
              if ( c > 0 )
                { for ( r = 0 ; r < NACT ; r++ )
                    { g += actsum[ r ];
                      switch ( r )
                        { case ACTSLEEP:   x = gdTrueColor(   0,  0,255 ); break;  // blue
                          case ACTEAT:     x = gdTrueColor(   0,255,  0 ); break;  // green
                          case ACTTURNCW:  x = gdTrueColor( 128,128,  0 ); break;  // orange-brown
                          case ACTTURNCCW: x = gdTrueColor( 128,  0,128 ); break;  // purple
                          case ACTMOVE:    x = gdTrueColor( 255,  0,  0 ); break;  // red
                          case ACTMATE:    x = gdTrueColor( 255,255,255 ); break;  // white
                          case ACTDIVIDE:  x = gdTrueColor(   0,255,255 ); break;  // cyan
                          case ACTMATED:   x = gdTrueColor( 128,  0,255 ); break;  // purple-blue
                          case ACTDEFEND:  x = gdTrueColor( 192,255,  0 ); break;  // yellow-green
                          default:         x = gdTrueColor( 255,255,255 ); break;
                        }
                      gdImageLine( im, WORLD_X + LEFTBAR + ( b * RIGHTBAR ) / c    , y,
                                       WORLD_X + LEFTBAR + ( g * RIGHTBAR ) / c - 1, y, x );
                      b = g;
                    }
                }
            }

          // Population density, age, weight down the leftbar
          maxage = maxbugs = maxmass = maxkills = maxgenes = 1;  // Prescan for relative scaling
	  mingenes = 1024000;
          for ( y = 0 ; y < WORLD_Y ; y++ )
            { age = bugs = mass = kills = genes = 0;
              for ( x = 0 ; x < WORLD_X ; x++ )
                { if ( world[x][y].bug != NULL )
                    { bugs++;
                      age   += today - world[x][y].bug->birthday;
                      mass  += world[x][y].bug->pos[0].weight;
                      kills += world[x][y].bug->kills;
                      genes += world[x][y].bug->brain.ngenes;
                    }
                }
              if ( bugs  < 1 ) bugs = 1;
              age   = (  age * 1024) / bugs;
              mass /= bugs;
              kills = (kills * 1024) / bugs;
              genes = (genes * 1024) / bugs;

              if ( bugs  > maxbugs  ) maxbugs  = bugs ;
              if ( age   > maxage   ) maxage   = age  ;
              if ( mass  > maxmass  ) maxmass  = mass ;
              if ( kills > maxkills ) maxkills = kills;
              if ( genes > maxgenes ) maxgenes = genes;
	      if ( genes > 0 ) { if ( genes < mingenes ) mingenes = genes; }
            }
          if ( mingenes >= maxgenes )
	    { maxgenes = mingenes + 1;
	       mingenes--;
	    }
          for ( y = 0 ; y < WORLD_Y ; y++ )                  // do the real plot
            { age = bugs = mass = kills = genes = 0;
              for ( x = 0 ; x < WORLD_X ; x++ )
                { if ( world[x][y].bug != NULL )
                    { bugs++;
                      age   += today - world[x][y].bug->birthday;
                      mass  += world[x][y].bug->pos[0].weight;
                      kills += world[x][y].bug->kills;
                      genes += world[x][y].bug->brain.ngenes;
                    }
                }
              if ( bugs  < 1 ) bugs = 1;
              age   = (  age * 1024) / bugs;
              mass /= bugs;
              kills = (kills * 1024) / bugs;
              genes = (genes * 1024) / bugs;
	      
	      if ( genes == 0 ) 
	        genes = mingenes;

              if ( y > 0 )
                { gdImageLine( im, (lastbugs  * LEFTBAR) / maxbugs , y - 1, (bugs  * LEFTBAR) / maxbugs , y, gdTrueColor( 255,255,  0 ) ); // Population in yellow
                  gdImageLine( im, (lastage   * LEFTBAR) / maxage  , y - 1, (age   * LEFTBAR) / maxage  , y, gdTrueColor( 255,255,255 ) ); // Age in white
                  gdImageLine( im, (lastmass  * LEFTBAR) / maxmass , y - 1, (mass  * LEFTBAR) / maxmass , y, gdTrueColor(   0,  0,255 ) ); // Mass in blue
                  gdImageLine( im, (lastkills * LEFTBAR) / maxkills, y - 1, (kills * LEFTBAR) / maxkills, y, gdTrueColor( 255,  0,  0 ) ); // Kills in red
                  gdImageLine( im, ((lastgenes - mingenes) * LEFTBAR) / (maxgenes - mingenes), y - 1,
		                                 ((genes - mingenes) * LEFTBAR) / (maxgenes - mingenes), y, gdTrueColor(   0,255,  0 ) ); // Genes in green
                }
              lastage  = age;       
              lastbugs = bugs;      
              lastmass = mass;      
              lastkills = kills;    
              lastgenes = genes;  
            }

          // Save the file
   	  jpegout = fopen( fn, "wb"); //open a file
	  gdImageJpeg( im, jpegout, 95); //write the image to the file using high quality setting
          fclose(jpegout);
}


int main() 
{       long  done,interval;
  gdImagePtr  im,imout; //declaration of the image
        char  fn[20],in[20];
        long  stage = 0;
        long  wait = 0;

  rot[0] =  988; // .966 @ 10, .342 @ 30  Go easy on the bug itself
  rot[1] =  973; //                       Heavier rot next door...
  rot[2] = 1012; //                       Very light farther on
  rot[3] = 1023; //                       Just a bit less than stopgrowth out here


  im    = gdImageCreateTrueColor(WORLD_X + SIDEBAR,WORLD_Y + BOTTOMBAR); //create an image
	// imout = gdImageCreateTrueColor( OUT_X, OUT_Y); //create a smaller output image???

  // for ( interval = 2 ; interval < 8 ; interval *= 2 )
  //   for ( done = 0 ; done < 64; done++ )
  //     printf("%ld - %ld\n",interval, limitedrandom( interval ) );

  init_world();
  bug_one();  // Load the original bug

  interval = 16;
  done = 0;
  while ( !done )
    { today++;

//      if ( today ==   8192 ) interval =   32;  // 16 sec
//      if ( today ==   9216 ) interval =   64;  // 1 sec
//      if ( today ==  11264 ) interval =  128;  // 1 sec
//      if ( today ==  15360 ) interval =  256;  // 1 sec
//      if ( today ==  23552 ) interval =  512;  // 1 sec
//      if ( today ==  39936 ) interval = 1024;  // 3 sec
//      if ( today == 131072 ) interval =  512;  // .5 sec
//      if ( today == 139264 ) interval =  256;  // .5 sec
//      if ( today == 143360 ) interval =  128;  // 5 sec
//      if ( today == 163840 ) interval =   64;  // .5 sec
//      if ( today == 164864 ) interval =   24;  // .5 sec
//      if ( today == 165376 ) interval =    8;  // 16 sec
//      if ( today == 173568 ) interval =   32;  // 1 sec
//      if ( today == 174592 ) interval =   64;  // 1 sec
//      if ( today == 176640 ) interval =  128;  // 1 sec
//      if ( today == 180736 ) interval =  256;  
//      if ( today == 262144 ) interval =  512;  // Begin year 8
//      if ( today == 294912 ) interval = 1024;  // Begin year 9

      // Dynamic challenges
      if ( wait > 0 )
        wait--;
       else
        { if (( stage == 0 ) && (buglist.n_bugs > 1000)) { foodhump = 10.0; stage = 1; wait = 0; }
	  if (( stage == 1 ) && (buglist.n_bugs > 10000)) { safety = 0; stage = 2; wait =   0; }
          if (( stage == 2 ) && (buglist.n_bugs > 15000)) { leak   = 0; stage = 3; wait = 250; }
          
          // Could use stage 3 & beyond for other things, but the forcemate escalation is somewhat population controlled using agediv


//          if (( stage == 1 ) && (buglist.n_bugs > 20000)) { leak = 1; stage = 2; wait = 100; }
//          if ( today == 4800 ) leak = 2;
//          if ( today == 9600 ) leak = 3;  // progressively less leaking going on around bugs
        }

      // if ( ( today % 256 ) == 0 )
      //   if ( today > SEASONLENGTH )
      //     if ( geneknee2 > 100 )     // Will reach 100 around "Year 9"
      //       geneknee2--;             // Increase the cost of carrying genes, very slowly, to evolve highly efficient bugs?

      if ( today ==  3000 ) forcemate = 0x10;  // Doesn't do anything until agediv is > 0 
      if ( today ==  4000 ) forcemate = 0x30;  // Start charging for frivilous underage divisions, and allow asexual divisions
      if ( today ==  5000 ) forcemate = 0x70;  // Start charging more for frivilous underage divisions, and allow asexual divisions
      if ( today ==  6000 ) forcemate = 0x71;  // Begin escalation of mating requirement
      if ( today ==  7000 ) forcemate = 0x73;
      if ( today ==  8000 ) forcemate = 0x77;  // Now start to force intelligent division decisions
      if ( today ==  9000 ) forcemate = 0x7F;
      if ( today == 10000 )  costmate =  24;   // Escalate costmate in stages
      if ( today == 11000 )  costmate =  48;
      if ( today == 12000 )  costmate =  96;
      if ( today == 13000 )  costmate = 144;

      if ( today > 3000 )
        { if ( today > SEASONLENGTH )
            { if ( ( today % 32 ) == 0 )
                { if ( agediv < 30 )
                    foodhump = foodhump * 1.001;
                  if ( agediv > 300 )
                    foodhump = foodhump / 1.001;
  
                // if ( today > SEASONLENGTH * 2 )
                //   { target_pop = (target_pop * 2047) / 2048;
                //     if ( target_pop < 3000 )
                //       target_pop = 30000;  // Will result in slow increase of food supply, not instant boom
                //   }

                }
              if (( today % SEASONLENGTH ) == 0 )             
                { if ( (( today / SEASONLENGTH ) % 2) == 0 )
                    forcemate = 0x70;  // return to allowing asexual division
                   else
                    forcemate = 0x7F;  // return to requiring sexual division
                }

              if ( buglist.n_bugs < 1000 )
                forcemate = 0x70;  // Have a heart, if heading to extinction, allow asexual division

            } // endif ( today > SEASONLENGTH )
  
//          if ((( buglist.n_bugs * 20 ) > ( WORLD_X * WORLD_Y )) && ( agediv < 2048 ))
          if (( buglist.n_bugs > target_pop * 2 ) && ( agediv < (today - buglist.first->birthday) ))  // Never increase beyond age of oldest bug
            agediv++;  // up-regulate when population is greater than 5% of available space      

          if ( ( today % 8 ) == 0 )
            agediv++; // Constantly attempt to up-regulate, will be caught & rtz later when < 16384 pop

        }

//      if ((( buglist.n_bugs * 40 ) < ( WORLD_X * WORLD_Y )) && ( agediv > 0 ))
      if ( (( buglist.n_bugs < target_pop ) && ( agediv > 0 )) || ( agediv > (today - buglist.first->birthday)) )
        agediv--;  // down-regulate when population is less than 2.5% of available space      

      if ( buglist.n_bugs > POP_HARDLIMIT )        // Agressive population control
        agediv = today - buglist.first->birthday;

      hist[today % LHIST].movement    =
      hist[today % LHIST].collisions  =
      hist[today % LHIST].starvations =
      hist[today % LHIST].births      = 0;  // counters incremented in move_bugs();

      move_bugs();
      grow_food();

      if ( buglist.n_bugs == 0 )
        { done = 1;
          printf("All bugs dead.\n");
        }
       else       
        { hist[today % LHIST].n_bugs    = buglist.n_bugs;
          hist[today % LHIST].avgweight = totalbug / buglist.n_bugs;
          hist[today % LHIST].avgfood   = (totalfood * 1024)/(WORLD_X * WORLD_Y);
          hist[today % LHIST].avggenes  = (genecount * 1024)/buglist.n_bugs;
 
          if ( today % 100 == 0 ) // in shell update
            printf( "%6ldDy %5ldBg %4.1f%% %8ld %8ld F=%5.0f B=%5.0f Gns=%6.2f AD%ld\n", today, buglist.n_bugs, ((float)(buglist.n_bugs * 100))/((float)(WORLD_X * WORLD_Y)), buglist.first->brain.eth.uid, idcounter, (float)totalfood/((float)WORLD_X * WORLD_Y), (float)totalbug/((float)buglist.n_bugs*1024),((float)genecount)/((float)buglist.n_bugs),agediv ); 
        }

      if ( today % (SEASONLENGTH / 8) == 0 )
        { sprintf( fn, "year%02ld%02ld.html", today / SEASONLENGTH, (today % SEASONLENGTH) / 1024 );
          sprintf( in, "year%02ld%02ld.jpg",  today / SEASONLENGTH, (today % SEASONLENGTH) / 1024 );
          bug_report( fn, in );  // Archival reports
          image_plot( im, in );
        }
          
      if ( today % 1000 == 0 )
        { bug_report( "newreport.html", "bugs.jpg" );  // Periodic reports for web update
          image_plot( im, "newbugs.jpg" );             // renamed to bugs.jpg by the script
        }
          
      if ( today % interval == 0 )        
        { sprintf( in, "b%07ld.jpg", today );  // images for later animation
          image_plot( im, in );                  
        }
 
    }  
  
   	//  jpegout = fopen( fn, "w"); //open a file
        //  gdImageCopyResampled( imout, im, 0,0,0,0, OUT_X, OUT_Y, PLOT_X, PLOT_Y);
	//  gdImageJpeg(imout, jpegout, -1); //write the image to the file using the default quality setting
        //  printf("%s %7.4f<x<%7.4f %7.4f<y<%7.4f\n",fn,minx,maxx,miny,maxy);
        //  fclose(jpegout); 

       

	/* be good, clean up stuff */
	gdImageDestroy(im);
	// gdImageDestroy(imout);
}

