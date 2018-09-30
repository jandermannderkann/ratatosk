﻿using System;
using DSACore.FireBase;
using DSALib;
using DSALib.Characters;
using Microsoft.EntityFrameworkCore.Design;

namespace DSACore.DSA_Game
{
    using System.Collections.Generic;
    using System.IO;
    using System.Linq;
    using DSACore.DSA_Game.Characters;
    using DSACore.DSA_Game.Save;

    public static class Dsa
    {
        public const string rootPath = "C:\\Users\\Dennis\\Source\\Repos\\DiscoBot\\DSACore\\";//"DiscoBot\\DSACore\\";

        private static Session s_session;

        public static List<ICharacter> Chars { get; set; } = new List<ICharacter>();  // list of all characters

        public static List<Talent> Talente { get; set; } = new List<Talent>();

        public static List<Zauber> Zauber { get; set; } = new List<Zauber>();

        public static Session Session
        {
            get
            {
                s_session.Chars = Chars.Select(x => SaveChar.FromICharacter(x)).ToList();
                return s_session;
            }

            set
            {
                s_session = value;
                foreach (var x in value.Chars)
                {
                    Chars.Find(c => c.Name.Equals(x.Name)).Update(x);
                }
            }
        }

        public static void start(){}

        public static void Startup()
        {
            //new .Auxiliary.Calculator.StringSolver("1d100 - (1d200 + 1) * -50000").Solve();
            /*Session = new Session();*/
            // relation.Add("Papo", "Pump aus der Gosse");
            foreach (var filename in Directory.GetFiles(rootPath + "helden", "*.xml"))
            {
                Chars.Add(new Character(filename));
                (Chars.Last() as Character)?.Talente.Select(x => new Talent(x.Name, x.Probe, 0))
                    .Where(c => !Talente.Exists(v => v.Name.Equals(c.Name))).ToList().ForEach(v => Talente.Add(v));
                (Chars.Last() as Character)?.Zauber.Select(x => new Zauber(x.Name, x.Probe, 0, x.Complexity))
                    .Where(c => !Zauber.Exists(v => v.Name.Equals(c.Name))).ToList().ForEach(v => Zauber.Add(v));
            }

            Properties.Deserialize(rootPath+"Properties");
            Properties.Serialize(rootPath + "Properties");

            Talente = Talente.OrderBy(x => x.Name).ToList();
            Zauber = Zauber.OrderBy(x => x.Name).ToList();

            /*foreach (var talent in Talente)
            {
                Database.AddTalent(new Models.Database.Talent(talent.Name, talent.Probe));
            }

            foreach (var talent in Zauber)
            {
                Database.AddSpell(new Models.Database.GeneralSpell(talent.Name, talent.Probe, talent.Complexity));
            }*/


            Session = new Session
            {
                Chars = Chars.Select(x => SaveChar.FromICharacter(x)).ToList()
            };
            //Session.Save();
        }

        public static ICharacter GetCharacter(ulong id)
        {
            throw new NotImplementedException();
        }

        public static ICharacter GetCharacter(string name, ulong groupId)
        {
            throw new NotImplementedException();
        }
    }
}