using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEngine;
using UnityEngine.TestTools;

namespace SpriteDicing.Test
{
    [ExcludeFromCoverage]
    public static class Helpers
    {
        public static class Paths
        {
            public static readonly string B = BuildTexturePath("1x1/B");
            public static readonly string R = BuildTexturePath("1x1/R");
            public static readonly string BGRT = BuildTexturePath("2x2/BGRT");
            public static readonly string BTGR = BuildTexturePath("2x2/BTGR");
            public static readonly string BTGT = BuildTexturePath("2x2/BTGT");
            public static readonly string TTTT = BuildTexturePath("2x2/TTTT");
            public static readonly string RGB1x3 = BuildTexturePath("RGB1x3");
            public static readonly string RGB3x1 = BuildTexturePath("RGB3x1");
            public static readonly string RGB4x4 = BuildTexturePath("RGB4x4");
            public static readonly string UIC4x4 = BuildTexturePath("UIC4x4");
            public static readonly IReadOnlyList<string> OneByOne = new[] { B, R };
            public static readonly IReadOnlyList<string> TwoByTwo = new[] { BGRT, BTGR, BTGT, TTTT };
            public static readonly IReadOnlyList<string> TopLevel = new[] { RGB1x3, RGB3x1, RGB4x4, UIC4x4 };
            public static readonly IReadOnlyList<string> All = TopLevel.Concat(TwoByTwo).Concat(OneByOne).ToArray();
        }

        public static class Textures
        {
            public static Texture2D B => LoadTexture(Paths.B);
            public static Texture2D R => LoadTexture(Paths.R);
            public static Texture2D BGRT => LoadTexture(Paths.BGRT);
            public static Texture2D BTGR => LoadTexture(Paths.BTGR);
            public static Texture2D BTGT => LoadTexture(Paths.BTGT);
            public static Texture2D TTTT => LoadTexture(Paths.TTTT);
            public static Texture2D RGB1x3 => LoadTexture(Paths.RGB1x3);
            public static Texture2D RGB3x1 => LoadTexture(Paths.RGB3x1);
            public static Texture2D RGB4x4 => LoadTexture(Paths.RGB4x4);
            public static Texture2D UIC4x4 => LoadTexture(Paths.UIC4x4);
        }

        public static class Colors
        {
            public static readonly Color32 Red = new(255, 0, 0, 255);
            public static readonly Color32 Green = new(0, 255, 0, 255);
            public static readonly Color32 Blue = new(0, 0, 255, 255);
            public static readonly Color32 Black = new(0, 0, 0, 255);
        }

        public const string TextureFolderPath = "Assets/Tests/Textures";

        public static string BuildTexturePath (string textureName)
        {
            return $"{TextureFolderPath}/{textureName}.png";
        }

        public static Texture2D LoadTexture (string texturePath)
        {
            return AssetDatabase.LoadAssetAtPath<Texture2D>(texturePath);
        }

        public static TextureImporter GetImporter (string texturePath)
        {
            return (TextureImporter)AssetImporter.GetAtPath(texturePath);
        }

        public static TextureImporter GetImporter (Texture2D texture)
        {
            var texturePath = AssetDatabase.GetAssetPath(texture);
            return GetImporter(texturePath);
        }
    }
}
