structstruck::strike! {
#[structstruck::each[derive(facet::Facet, smart_default::SmartDefault)]]
#[structstruck::each[facet(rename_all = "camelCase", deny_unknown_fields)]]
pub struct SystemConfig {
    backup: Database,
    ffmpeg: Ffmpeg,
    logging: Logging,
    machine_learning: MachineLearning,
  map: struct {
    #[default = true]
    enabled: bool,
    #[default = "https://tiles.immich.cloud/v1/style/light.json"]
    light_style: String,
    #[default = "https://tiles.immich.cloud/v1/style/dark.json"]
    dark_style: String
  },
  new_version_check: struct {
    #[default = true]
    enabled: bool
  },
  nightly_tasks: struct {
    #[default = "00:00"]
    start_time: String,
    #[default = true]
    database_cleanup: bool,
    #[default = true]
    missing_thumbnails: bool,
    #[default = true]
    cluster_new_faces: bool,
    #[default = true]
    generate_memories: bool,
    #[default = true]
    sync_quota_usage: bool
  },
  oauth: struct {
    #[default =  false]
    auto_launch: bool,
    #[default =  true]
    auto_register: bool,
    #[default =  "Login" ]
    button_text: String,
    #[default =  ""]
    client_id: String,
    #[default =  ""]
    client_secret: String,
    #[default =  "client_secret_post"]
    token_endpoint_auth_method: String,
    #[default =  30000]
    timeout: usize,
    #[default(None)]
    default_storage_quota: Option<usize>,
    #[default =  false]
    enabled: bool,
    #[default =  ""]
    issuer_url: String,
    #[default =  false]
    mobile_override_enabled: bool,
    #[default =  ""]
    mobile_redirect_uri: String,
    #[default =  "openid" ]
    scope: String,
    #[default =  "RS256"]
    signing_algorithm: String,
    #[default =  "none"]
    profile_signing_algorithm: String,
    #[default =  "preferred_username"]
    storage_label_claim: String,
    #[default =  "immich_quota"]
    storage_quota_claim: String,
    #[default =  "immich_role" ]
    role_claim: String,
  },
  password_login: struct {
    #[default =  true]
    enabled: bool,
  },
  reverse_geocoding: struct {
    #[default =  true]
    enabled: bool,
  },
  metadata: struct {
    faces: struct {
      #[default =  true]
      import: bool,
    }
  },
  storage_template: struct {
    #[default = false]
    enabled: bool,
    #[default = true]
    hash_verification_enabled: bool,
    #[default = "{{y}}/{{y}}-{{MM}}-{{dd}}/{{filename}}"]
    template: String,
  },
  job: struct {
    thumbnail_generation: struct JobThumbnailGeneration {
      #[default = 3]
      concurrency: usize,
    },
    metadata_extraction: struct JobMetadataExtraction {
      #[default = 5]
      concurrency: usize,
    },
    video_conversion: struct JobVideoConversion {
      #[default = 1]
      concurrency: usize,
    },
    smart_search: struct JobSmartSearch {
      #[default = 1]
      concurrency: usize,
    },
    migration: struct JobMigration {
      #[default = 1]
      concurrency: usize,
    },
    background_task: struct JobBackgroundTask {
      #[default = 1]
      concurrency: usize,
    },
    search: struct JobSearch {
      #[default = 1]
      concurrency: usize,
    },
    face_detection: struct JobFaceDetection {
      #[default = 1]
      concurrency: usize,
    },
    ocr: struct JobOcr {
      #[default = 1]
      concurrency: usize,
    },
    sidecar: struct JobSidecar {
      #[default = 1]
      concurrency: usize,
    },
    library: struct JobLibrary {
      #[default = 1]
      concurrency: usize,
    },
    notifications: struct JobNotifications {
      #[default = 1]
      concurrency: usize,
    },
    workflow: struct JobWorkflow {
      #[default = 1]
      concurrency: usize,
    },
    editor: struct JobEditor {
      #[default = 1]
      concurrency: usize,
    }
  },
  image: struct {
    thumbnail: struct {
      #[default = "webp"]
      format: String,
      #[default = 80]
      quality: usize,
      #[default = 250]
      size: usize,
      #[default = false]
      progressive: bool,
    },
    preview: struct {
      #[default = "jpeg"]
      format: String,
      #[default = 80]
      quality: usize,
      #[default = 1440]
      size: usize,
      #[default = false]
      progressive: bool,
    },
    fullsize: struct {
      #[default = false]
      enabled: bool,
      #[default = "jpeg"]
      format: String,
      #[default = 80]
      quality: usize,
      #[default = false]
      progressive: bool,
    },
    #[default = "p3"]
    colorspace: String,
    #[default = false]
    extract_embedded: bool,
  },
  trash: struct {
    #[default = true]
    enabled: bool,
    #[default = 30]
    days: usize,
  },
  theme: struct {
    #[default = ""]
    custom_css: String,
  },
  library: struct {
    scan: struct {
      #[default = true]
      enabled: bool,
      #[default = "0 0 * * *"]
      cron_expression: String,
    },
    watch: struct {
      #[default = true]
      enabled: bool,
    }
  },
  notifications: struct {
    smtp: struct {
      enabled: bool,
      from: String,
      reply_to: String,
      transport: struct {
        ignore_cert: bool,
        host: String,
        port: usize,
        secure: bool,
        username: String,
        password: String,
      }
    }
  },
  templates: struct {
    email: struct {
      album_invite_template: String,
      welcome_template: String,
      album_update_template: String,
    }
  },
  server: struct {
    external_domain: String,
    login_page_message: String,
    #[default = true]
    public_users: bool,
  },
  user: struct {
    #[default = 7]
    delete_delay: usize
  }
}
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Backup {
    database: Database,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Database {
    #[default = true]
    enabled: bool,
    #[default = "0 02 * * *"]
    cron_expression: String,
    #[default = 14]
    keep_last_amount: usize,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Ffmpeg {
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

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Logging {
    #[default = true]
    enabled: bool,
    #[default = "log"]
    level: String,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct MachineLearning {
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

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct AvailabilityChecks {
    #[default = true]
    enabled: bool,
    #[default = 2000]
    timeout: usize,
    #[default = 3000]
    interval: usize,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Clip {
    #[default = true]
    enabled: bool,
    #[default = "ViT-B-32__openai"]
    model_name: String,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct DuplicateDetection {
    #[default = true]
    enabled: bool,
    #[default = 0.01]
    max_distance: f64,
}

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct FacialRecognition {
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

#[derive(facet::Facet, smart_default::SmartDefault)]
#[facet(rename_all = "camelCase", deny_unknown_fields)]
struct Ocr {
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
