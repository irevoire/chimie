structstruck::strike! {
#[structstruck::each[derive(Debug, facet::Facet, smart_default::SmartDefault)]]
#[structstruck::each[facet(rename_all = "camelCase", deny_unknown_fields)]]
pub struct SystemConfig {
    pub backup: Database,
    pub ffmpeg: Ffmpeg,
    pub logging: Logging,
    pub machine_learning: MachineLearning,
  pub map: struct {
    #[default = true]
    pub enabled: bool,
    #[default = "https://tiles.immich.cloud/v1/style/light.json"]
    pub light_style: String,
    #[default = "https://tiles.immich.cloud/v1/style/dark.json"]
    pub dark_style: String
  },
  pub new_version_check: struct {
    #[default = true]
    pub enabled: bool
  },
  pub nightly_tasks: struct {
    #[default = "00:00"]
    pub start_time: String,
    #[default = true]
    pub database_cleanup: bool,
    #[default = true]
    pub missing_thumbnails: bool,
    #[default = true]
    pub cluster_new_faces: bool,
    #[default = true]
    pub generate_memories: bool,
    #[default = true]
    pub sync_quota_usage: bool
  },
  pub oauth: struct {
    #[default =  false]
    pub auto_launch: bool,
    #[default =  true]
    pub auto_register: bool,
    #[default =  "Login" ]
    pub button_text: String,
    #[default =  ""]
    pub client_id: String,
    #[default =  ""]
    pub client_secret: String,
    #[default =  "client_secret_post"]
    pub token_endpoint_auth_method: String,
    #[default =  30000]
    pub timeout: usize,
    #[default(None)]
    pub default_storage_quota: Option<usize>,
    #[default =  false]
    pub enabled: bool,
    #[default =  ""]
    pub issuer_url: String,
    #[default =  false]
    pub mobile_override_enabled: bool,
    #[default =  ""]
    pub mobile_redirect_uri: String,
    #[default =  "openid" ]
    pub scope: String,
    #[default =  "RS256"]
    pub signing_algorithm: String,
    #[default =  "none"]
    pub profile_signing_algorithm: String,
    #[default =  "preferred_username"]
    pub storage_label_claim: String,
    #[default =  "immich_quota"]
    pub storage_quota_claim: String,
    #[default =  "immich_role" ]
    pub role_claim: String,
  },
  pub password_login: struct {
    #[default =  true]
    pub enabled: bool,
  },
  pub reverse_geocoding: struct {
    #[default =  true]
    pub enabled: bool,
  },
  pub metadata: struct {
    pub faces: struct {
      #[default =  true]
      pub import: bool,
    }
  },
  pub storage_template: StorageTemplate,
  pub job: struct {
    pub thumbnail_generation: struct JobThumbnailGeneration {
      #[default = 3]
      pub concurrency: usize,
    },
    pub metadata_extraction: struct JobMetadataExtraction {
      #[default = 5]
      pub concurrency: usize,
    },
    pub video_conversion: struct JobVideoConversion {
      #[default = 1]
      pub concurrency: usize,
    },
    pub smart_search: struct JobSmartSearch {
      #[default = 1]
      pub concurrency: usize,
    },
    pub migration: struct JobMigration {
      #[default = 1]
      pub concurrency: usize,
    },
    pub background_task: struct JobBackgroundTask {
      #[default = 1]
      pub concurrency: usize,
    },
    pub search: struct JobSearch {
      #[default = 1]
      pub concurrency: usize,
    },
    pub face_detection: struct JobFaceDetection {
      #[default = 1]
      pub concurrency: usize,
    },
    pub ocr: struct JobOcr {
      #[default = 1]
      pub concurrency: usize,
    },
    pub sidecar: struct JobSidecar {
      #[default = 1]
      pub concurrency: usize,
    },
    pub library: struct JobLibrary {
      #[default = 1]
      pub concurrency: usize,
    },
    pub notifications: struct JobNotifications {
      #[default = 1]
      pub concurrency: usize,
    },
    pub workflow: struct JobWorkflow {
      #[default = 1]
      pub concurrency: usize,
    },
    pub editor: struct JobEditor {
      #[default = 1]
      pub concurrency: usize,
    }
  },
  pub image: struct {
    pub thumbnail: struct {
      #[default = "webp"]
      pub format: String,
      #[default = 80]
      pub quality: usize,
      #[default = 250]
      pub size: usize,
      #[default = false]
      pub progressive: bool,
    },
    pub preview: struct {
      #[default = "jpeg"]
      pub format: String,
      #[default = 80]
      pub quality: usize,
      #[default = 1440]
      pub size: usize,
      #[default = false]
      pub progressive: bool,
    },
    pub fullsize: struct {
      #[default = false]
      pub enabled: bool,
      #[default = "jpeg"]
      pub format: String,
      #[default = 80]
      pub quality: usize,
      #[default = false]
      pub progressive: bool,
    },
    #[default = "p3"]
    pub colorspace: String,
    #[default = false]
    pub extract_embedded: bool,
  },
  pub trash: struct {
    #[default = true]
    pub enabled: bool,
    #[default = 30]
    pub days: usize,
  },
  pub theme: struct {
    #[default = ""]
    pub custom_css: String,
  },
  pub library: struct {
    pub scan: struct {
      #[default = true]
      pub enabled: bool,
      #[default = "0 0 * * *"]
      pub cron_expression: String,
    },
    pub watch: struct {
      #[default = true]
      pub enabled: bool,
    }
  },
  pub notifications: struct {
    pub smtp: struct {
      pub enabled: bool,
      pub from: String,
      pub reply_to: String,
      pub transport: struct {
        pub ignore_cert: bool,
        pub host: String,
        pub port: usize,
        pub secure: bool,
        pub username: String,
        pub password: String,
      }
    }
  },
  pub templates: struct {
    pub email: struct {
      pub album_invite_template: String,
      pub welcome_template: String,
      pub album_update_template: String,
    }
  },
  pub server: struct {
    pub external_domain: String,
    pub login_page_message: String,
    #[default = true]
    pub public_users: bool,
  },
  pub user: struct {
    #[default = 7]
    pub delete_delay: usize
  }
}
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct StorageTemplate {
    #[default = false]
    enabled: bool,
    #[default = true]
    hash_verification_enabled: bool,
    #[default = "{{y}}/{{y}}-{{MM}}-{{dd}}/{{filename}}"]
    template: String,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Backup {
    database: Database,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Database {
    #[default = true]
    enabled: bool,
    #[default = "0 02 * * *"]
    cron_expression: String,
    #[default = 14]
    keep_last_amount: usize,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ffmpeg {
    #[default = 23]
    crf: usize,
    #[default = 0]
    threads: usize,
    #[default = "ultrafast"]
    preset: String,
    #[default = "h264"]
    target_video_codec: String,
    #[default(vec![String::from("h264")])]
    accepted_video_codecs: Vec<String>,
    #[default = "aac"]
    target_audio_codec: String,
    #[default(vec![String::from("aac"), String::from("mp3"), String::from("libopus")])]
    accepted_audio_codecs: Vec<String>,
    #[default(vec![String::from("mov"), String::from("ogg"), String::from("webm")])]
    accepted_containers: Vec<String>,
    #[default = "720"]
    target_resolution: String,
    #[default = "0"]
    max_bitrate: String,
    #[default(-1)]
    bframes: isize,
    #[default = 0]
    refs: usize,
    #[default = 0]
    gop_size: usize,
    #[default = false]
    temporal_a_q: bool,
    #[default = "auto"]
    cq_mode: String,
    #[default = false]
    two_pass: bool,
    #[default = "auto"]
    preferred_hw_device: String,
    #[default = "required"]
    transcode: String,
    #[default = "disabled"]
    accel: String,
    #[default = false]
    accel_decode: bool,
    #[default = "hable"]
    tonemap: String,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Logging {
    #[default = true]
    enabled: bool,
    #[default = "log"]
    level: String,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct MachineLearning {
    #[default = false]
    enabled: bool,
    #[default(vec![])]
    urls: Vec<String>,
    availability_checks: AvailabilityChecks,
    clip: Clip,
    duplicate_detection: DuplicateDetection,
    facial_recognition: FacialRecognition,
    ocr: Ocr,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvailabilityChecks {
    #[default = true]
    enabled: bool,
    #[default = 2000]
    timeout: usize,
    #[default = 3000]
    interval: usize,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Clip {
    #[default = true]
    enabled: bool,
    #[default = "ViT-B-32__openai"]
    model_name: String,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct DuplicateDetection {
    #[default = true]
    enabled: bool,
    #[default = 0.01]
    max_distance: f64,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct FacialRecognition {
    #[default = true]
    enabled: bool,
    #[default = "buffalo_l"]
    model_name: String,
    #[default = 0.7]
    min_score: f64,
    #[default = 0.5]
    max_distance: f64,
    #[default = 3]
    min_faces: usize,
}

#[derive(Debug, facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
pub struct Ocr {
    #[default = true]
    enabled: bool,
    #[default = "PP-OCRv5_mobile"]
    model_name: String,
    #[default = 736]
    max_resolution: usize,
    #[default = 0.5]
    min_detection_score: f64,
    #[default = 0.8]
    min_recognition_score: f64,
}
