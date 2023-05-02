use chrono::{DateTime, Local, Utc};
use ring::digest::{Context, SHA256};
use std::fs::File;
use std::io;
use std::io::Write;

// Définition d'une structure Personne pour stocker les données de chaque personne
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Personne {
    nom: String,
    prenom: String,
    badge: u32,
    code_secret: String,
    question_secrete: String,
    reponse_secrete: String,
    dernier_passage: Option<DateTime<Utc>>,
    tentatives: u8,
}

impl Personne {
    // Fonction pour afficher les données d'une personne
    fn afficher(&self) {
        println!("Nom : {}", self.nom);
        println!("Prénom : {}", self.prenom);
        println!("Numéro de badge : {}", self.badge);
        println!("Code secret : {}", self.code_secret);
        match self.dernier_passage {
            Some(date) => println!("Dernier passage : {}", date.to_rfc2822()),
            None => println!("Dernier passage : Aucun"),
        }
        println!("---------------------");
    }
}

fn main() {
    let mut personnes: Vec<Personne> = Vec::new();

    loop {
        println!("Menu :");
        println!("1. Afficher la liste des personnes");
        println!("2. Ajouter une personne");
        println!("3. Supprimer une personne");
        println!("4. Modifier le code secret d'une personne");
        println!("5. Simuler le contrôle d'accès");
        println!("6. Sauvegarder la liste des personnes dans un fichier au fomart CSV");
        println!("7. Quitter");

        // Lecture du choix de l'utilisateur
        let choix = saisir_entier("Choix :");
        match choix {
            1 => {
                println!("Liste des personnes :");
                for personne in &personnes {
                    personne.afficher();
                }
            }
            2 => {
                // Saisie des données de la personne
                let nom = saisir_chaine("Nom :");
                let prenom = saisir_chaine("Prénom :");
                let badge = saisir_entier("Numéro de badge (4 chiffres) :");
                let code_secret = saisir_chaine("Code secret :");
                let code_secret_hash = hash_code_secret(&code_secret);
                let question_secrete = saisir_chaine("Question secrète :");
                let reponse_secrete = saisir_chaine("Réponse secrète :");

                // Création d'une nouvelle personne
                let personne = Personne {
                    nom: nom,
                    prenom: prenom,
                    badge: badge,
                    code_secret: code_secret_hash,
                    question_secrete: question_secrete,
                    reponse_secrete: reponse_secrete,
                    dernier_passage: None,
                    tentatives: 0,
                };

                // Ajout de la personne à la liste
                personnes.push(personne);

                println!("Personne ajoutée avec succès !");
            }

            3 => {
                let badge = saisir_entier("Numéro de badge de la personne à supprimer :");
                let index = personnes.iter().position(|p| p.badge == badge);

                match index {
                    Some(i) => {
                        personnes.remove(i);
                        println!("Personne supprimée avec succès !");
                    }
                    None => println!("Aucune personne correspondante trouvée."),
                }
            }
            4 => {
                let badge = saisir_entier("Numéro de badge de la personne à modifier :");
                let index = personnes.iter().position(|p| p.badge == badge);

                match index {
                    Some(i) => {
                        let nouveau_code_secret = saisir_chaine("Nouveau code secret :");
                        let nouveau_code_secret_hash = hash_code_secret(&nouveau_code_secret);
                        personnes[i].code_secret = nouveau_code_secret_hash;
                        println!("Code secret modifié avec succès !");
                    }
                    None => println!("Aucune personne correspondante trouvée."),
                }
            }
            5 => {
                let badge = saisir_entier("Numéro de badge :");
                let index = personnes.iter().position(|p| p.badge == badge);

                match index {
                    Some(i) => {
                        if personnes[i].tentatives < 3 {
                            let code_secret = saisir_chaine("Code secret :");
                            let code_secret_hash = hash_code_secret(&code_secret);

                            if personnes[i].code_secret == code_secret_hash {
                                println!(
                                    "Bienvenue {} {} !",
                                    personnes[i].prenom, personnes[i].nom
                                );

                                let date = Local::now();
                                personnes[i].dernier_passage = Some(date.into());

                                println!("Accès autorisé le {}.", date.to_rfc2822());
                                personnes[i].tentatives = 0;
                            } else {
                                println!("Accès refusé !");
                                personnes[i].tentatives += 1;
                            }
                        } else {
                            let reponse_secrete = saisir_chaine(
                                "Trop de tentatives. Veuillez répondre à la question secrète :",
                            );
                            if reponse_secrete == personnes[i].reponse_secrete {
                                personnes[i].tentatives = 0;
                                println!("Compte débloqué.");
                            } else {
                                println!("Réponse incorrecte. Accès refusé.");
                            }
                        }
                    }
                    None => println!("Aucune personne correspondante trouvée."),
                }
            }
            6 => {
                let nom_fichier = saisir_chaine("Nom du fichier de sauvegarde :");
                match sauvegarder_personnes(&personnes, &nom_fichier) {
                    Ok(_) => println!("Liste sauvegardée dans {}.", nom_fichier),
                    Err(e) => eprintln!("Erreur : {}", e),
                }
            }
            7 => {
                println!("Au revoir !");
                break;
            }
            _ => println!("Choix invalide"),
        }
    }
}

// Fonction pour hasher un code secret
fn hash_code_secret(code_secret: &str) -> String {
    let mut context = Context::new(&SHA256);
    context.update(code_secret.as_bytes());
    let digest = context.finish();

    hex::encode(digest.as_ref())
}

// Fonction pour sauvegarder la liste de personnes dans un fichier CSV
fn sauvegarder_personnes(personnes: &[Personne], nom_fichier: &str) -> io::Result<()> {
    let mut fichier = File::create(nom_fichier)?;

    // Écrire l'en-tête du fichier CSV
    writeln!(
        fichier,
        "Nom, Prénom, Numéro de badge, Code secret, Dernier passage"
    )?;

    // Écrire les données de chaque personne dans le fichier CSV
    for personne in personnes {
        let dernier_passage = personne
            .dernier_passage
            .map(|d| d.to_rfc2822())
            .unwrap_or_default();
        writeln!(
            fichier,
            "{},{},{},{},{}",
            personne.nom, personne.prenom, personne.badge, personne.code_secret, dernier_passage
        )?;
    }

    Ok(())
}

// Fonction pour saisir une chaîne de caractères
fn saisir_chaine(message: &str) -> String {
    println!("{}", message);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Erreur de saisie");
    input.trim().to_string()
}

// Fonction pour saisir un entier
fn saisir_entier(message: &str) -> u32 {
    loop {
        let input = saisir_chaine(message);

        match input.parse::<u32>() {
            Ok(n) => return n,
            Err(_) => println!("Erreur : saisie invalide !"),
        }
    }
}
