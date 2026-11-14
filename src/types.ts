export interface SystemStatus {
  has_gpg: boolean;
  has_pass: boolean;
  has_pass_store: boolean;
  home_dir: string;
  pkg_manager: string;
}

export interface GpgKey {
  /** Long key id, e.g. the value passed to `pass init`. */
  id: string;
  /** Human readable "Name <email>". */
  uid: string;
}
